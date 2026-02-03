import {
  type Frame,
  type Rect,
  type ThirdPartyWidgetSettings,
  type Widget as IWidget,
  type WidgetConfigDefinition,
  type WidgetId,
  WidgetPreset,
  type WidgetSettingItem,
  WidgetStatus,
  type WidgetTriggerPayload,
} from "@seelen-ui/types";
import { invoke, SeelenCommand, SeelenEvent } from "../../handlers/mod.ts";
import { getCurrentWebviewWindow, type WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { decodeBase64Url } from "@std/encoding";
import { debounce } from "../../utils/async.ts";
import { WidgetAutoSizer } from "./sizing.ts";
import { adjustPositionByPlacement, fitIntoMonitor, initMonitorsState } from "./positioning.ts";
import { startThemingTool } from "../theme/theming.ts";
import type { InitWidgetOptions, WidgetInformation } from "./interfaces.ts";

interface WidgetInternalState {
  hwnd: number;
  initialized: boolean;
  ready: boolean;
  position: {
    x: number;
    y: number;
  };
  size: {
    width: number;
    height: number;
  };
}

/**
 * Represents the widget instance running in the current webview
 */
export class Widget {
  /**
   * Alternative accesor for the current running widget.\
   * Will throw if the library is being used on a non Seelen UI environment
   */
  static getCurrent(): Widget {
    const scope = globalThis as ExtendedGlobalThis;
    if (!scope.__SLU_WIDGET) {
      throw new Error("The library is being used on a non Seelen UI environment");
    }
    return (
      scope.__SLU_WIDGET_INSTANCE || (scope.__SLU_WIDGET_INSTANCE = new Widget(scope.__SLU_WIDGET))
    );
  }

  /** The current running widget */
  static get self(): Widget {
    return Widget.getCurrent();
  }

  /** widget id */
  public readonly id: WidgetId;
  /** widget definition */
  public readonly def: IWidget;
  /** decoded widget instance information */
  public readonly decoded: WidgetInformation;
  /** current webview window */
  public readonly webview: WebviewWindow;

  private autoSizer?: WidgetAutoSizer;
  private initOptions: InitWidgetOptions = {};

  private runtimeState: WidgetInternalState = {
    hwnd: 0,
    initialized: false,
    ready: false,
    position: {
      x: 0,
      y: 0,
    },
    size: {
      width: 0,
      height: 0,
    },
  };

  private constructor(widget: IWidget) {
    this.def = widget;
    this.webview = getCurrentWebviewWindow();

    const [id, query] = getDecodedWebviewLabel();
    const params = new URLSearchParams(query);
    const paramsObj = Object.freeze(Object.fromEntries(params));

    this.id = id as WidgetId;
    this.decoded = Object.freeze({
      label: `${id}${query ? `?${query}` : ""}`,
      monitorId: paramsObj.monitorId || null,
      instanceId: paramsObj.instanceId || null,
      params: Object.freeze(Object.fromEntries(params)),
    });
  }

  /** Returns the current window id of the widget */
  get windowId(): number {
    return this.runtimeState.hwnd;
  }

  /** Returns the current position and size of the widget */
  get frame(): Frame {
    return {
      x: this.runtimeState.position.x,
      y: this.runtimeState.position.y,
      width: this.runtimeState.size.width,
      height: this.runtimeState.size.height,
    };
  }

  /** Returns the default config of the widget, declared on the widget definition */
  public getDefaultConfig(): ThirdPartyWidgetSettings {
    const config: ThirdPartyWidgetSettings = { enabled: true };
    for (const definition of this.def.settings) {
      Object.assign(config, getDefinitionDefaultValues(definition));
    }
    return config;
  }

  private applyInvisiblePreset(): Array<Promise<void>> {
    return [
      this.webview.setDecorations(false), // no title bar
      this.webview.setShadow(false), // no shadows
      // hide from native shell
      this.webview.setSkipTaskbar(true),
      // as a (desktop/overlay) widget we don't wanna allow nothing of these
      this.webview.setMinimizable(false),
      this.webview.setMaximizable(false),
      this.webview.setClosable(false),
    ];
  }

  /** Will apply the recommended settings for a desktop widget */
  private async applyDesktopPreset(): Promise<void> {
    await Promise.all([...this.applyInvisiblePreset(), this.webview.setAlwaysOnBottom(true)]);
  }

  /** Will apply the recommended settings for an overlay widget */
  private async applyOverlayPreset(): Promise<void> {
    await Promise.all([...this.applyInvisiblePreset(), this.webview.setAlwaysOnTop(true)]);
  }

  /** Will apply the recommended settings for a popup widget */
  private async applyPopupPreset(): Promise<void> {
    await Promise.all([...this.applyInvisiblePreset(), this.webview.setAlwaysOnTop(true)]);

    const hideWebview = debounce(() => {
      this.hide(true);
    }, 100);

    this.webview.onFocusChanged(({ payload: focused }) => {
      if (focused) {
        hideWebview.cancel();
      } else {
        hideWebview();
      }
    });

    this.onTrigger(async ({ desiredPosition, alignX, alignY }) => {
      // avoid flickering when clicking a button that triggers the widget
      hideWebview.cancel();

      if (this.autoSizer && alignX && alignY) {
        this.autoSizer.originX = alignX;
        this.autoSizer.originY = alignY;
      }

      if (desiredPosition) {
        const adjusted = adjustPositionByPlacement({
          frame: {
            x: desiredPosition.x,
            y: desiredPosition.y,
            width: this.runtimeState.size.width,
            height: this.runtimeState.size.height,
          },
          originX: alignX,
          originY: alignY,
        });

        await this.setPosition({
          left: adjusted.x,
          top: adjusted.y,
          right: adjusted.x + adjusted.width,
          bottom: adjusted.y + adjusted.height,
        });
      }

      await this.show();
      await this.focus();
    });
  }

  /**
   * Will restore the saved position and size of the widget on start,
   * after that will store the position and size of the widget on change.
   */
  public async persistPositionAndSize(): Promise<void> {
    const storage = globalThis.window.localStorage;

    const [x, y, width, height] = [`x`, `y`, `width`, `height`].map((k) => storage.getItem(`${k}`));
    if (x && y && width && height) {
      const frame = {
        x: Number(x),
        y: Number(y),
        width: Number(width),
        height: Number(height),
      };

      const safeFrame = fitIntoMonitor(frame);
      await this.setPosition({
        left: safeFrame.x,
        top: safeFrame.y,
        right: safeFrame.x + safeFrame.width,
        bottom: safeFrame.y + safeFrame.height,
      });
    }

    this.webview.onMoved(
      debounce((e) => {
        const { x, y } = e.payload;
        storage.setItem(`x`, x.toString());
        storage.setItem(`y`, y.toString());
        console.info(`Widget position saved: ${x} ${y}`);
      }, 500),
    );

    this.webview.onResized(
      debounce((e) => {
        const { width, height } = e.payload;
        storage.setItem(`width`, width.toString());
        storage.setItem(`height`, height.toString());
        console.info(`Widget size saved: ${width} ${height}`);
      }, 500),
    );
  }

  /**
   * Will initialize the widget based on the preset and mark it as `pending`, this function won't show the widget.
   * This should be called before any other action on the widget. After this you should call
   * `ready` to mark the widget as ready and show it.
   */
  public async init(options: InitWidgetOptions = {}): Promise<void> {
    if (this.runtimeState.initialized) {
      console.warn(`Widget already initialized`);
      return;
    }

    this.runtimeState.hwnd = await invoke(SeelenCommand.GetSelfWindowId);

    this.runtimeState.initialized = true;
    this.initOptions = options;

    if (options.autoSizeByContent) {
      this.autoSizer = new WidgetAutoSizer(this, options.autoSizeByContent);
    } else if (options.saveAndRestoreLastRect ?? this.def.preset === WidgetPreset.Desktop) {
      await this.persistPositionAndSize();
    }

    switch (this.def.preset) {
      case WidgetPreset.None:
        break;
      case WidgetPreset.Desktop:
        await this.applyDesktopPreset();
        break;
      case WidgetPreset.Overlay:
        await this.applyOverlayPreset();
        break;
      case WidgetPreset.Popup:
        await this.applyPopupPreset();
        break;
    }

    await startThemingTool();
    await initMonitorsState();

    // state initialization
    this.runtimeState.size = await this.webview.outerSize();
    this.runtimeState.position = await this.webview.outerPosition();

    this.webview.onResized((e) => {
      this.runtimeState.size.width = e.payload.width;
      this.runtimeState.size.height = e.payload.height;
    });

    this.webview.onMoved((e) => {
      this.runtimeState.position.x = e.payload.x;
      this.runtimeState.position.y = e.payload.y;
    });
  }

  /**
   * Will mark the widget as `ready` and pool pending triggers.
   *
   * If the widget is not lazy this will inmediately show the widget.
   * Lazy widget should be shown on trigger action.
   */
  public async ready(): Promise<void> {
    if (!this.runtimeState.initialized) {
      throw new Error(`Widget was not initialized before ready`);
    }

    if (this.runtimeState.ready) {
      console.warn(`Widget is already ready`);
      return;
    }

    this.runtimeState.ready = true;
    await this.autoSizer?.execute();

    if (this.initOptions.show ?? !this.def.lazy) {
      await this.show();
    }

    // this will mark the widget as ready, and send pending trigger event if exists
    await invoke(SeelenCommand.SetCurrentWidgetStatus, { status: WidgetStatus.Ready });
  }

  public onTrigger(cb: (args: WidgetTriggerPayload) => void): void {
    this.webview.listen<WidgetTriggerPayload>(SeelenEvent.WidgetTriggered, ({ payload }) => {
      cb(payload);
    });
  }

  /** If animations are enabled this will animate the movement of the widget */
  public setPosition(rect: Rect): Promise<void> {
    this.runtimeState.position.x = rect.left;
    this.runtimeState.position.y = rect.top;
    this.runtimeState.size.width = rect.right - rect.left;
    this.runtimeState.size.height = rect.bottom - rect.top;

    return invoke(SeelenCommand.SetSelfPosition, {
      rect: {
        left: Math.round(rect.left),
        top: Math.round(rect.top),
        right: Math.round(rect.right),
        bottom: Math.round(rect.bottom),
      },
    });
  }

  public async show(): Promise<void> {
    debouncedClose.cancel();
    await this.webview.show();
  }

  /** Will force foreground the widget */
  public async focus(): Promise<void> {
    await invoke(SeelenCommand.RequestFocus, { hwnd: this.runtimeState.hwnd }).catch(() => {
      console.warn(`Failed to focus widget: ${this.decoded.label}`);
    });
  }

  public hide(closeAfterInactivity?: boolean): void {
    Widget.self.webview.hide();
    if (closeAfterInactivity) {
      debouncedClose();
    }
  }
}

const debouncedClose = debounce(() => {
  Widget.self.webview.close();
}, 30_000);

type ExtendedGlobalThis = typeof globalThis & {
  __SLU_WIDGET?: IWidget;
  __SLU_WIDGET_INSTANCE?: Widget;
};

export const SeelenSettingsWidgetId: WidgetId = "@seelen/settings" as WidgetId;
export const SeelenPopupWidgetId: WidgetId = "@seelen/popup" as WidgetId;
export const SeelenWegWidgetId: WidgetId = "@seelen/weg" as WidgetId;
export const SeelenToolbarWidgetId: WidgetId = "@seelen/fancy-toolbar" as WidgetId;
export const SeelenWindowManagerWidgetId: WidgetId = "@seelen/window-manager" as WidgetId;
export const SeelenWallWidgetId: WidgetId = "@seelen/wallpaper-manager" as WidgetId;

function getDecodedWebviewLabel(): [WidgetId, string | undefined] {
  const encondedLabel = getCurrentWebviewWindow().label;
  const decodedLabel = new TextDecoder().decode(decodeBase64Url(encondedLabel));
  const [id, query] = decodedLabel.split("?");
  if (!id) {
    throw new Error("Missing widget id on webview label");
  }
  return [id as WidgetId, query];
}

function getDefinitionDefaultValues(definition: WidgetConfigDefinition): Record<string, unknown> {
  const config: Record<string, unknown> = {};

  // Check if it's a group (has "group" property)
  if ("group" in definition) {
    // Recursively process all items in the group
    for (const item of definition.group.items) {
      Object.assign(config, getDefinitionDefaultValues(item));
    }
  } else {
    // It's a setting item, extract key and defaultValue
    const item = definition as WidgetSettingItem;
    if ("key" in item && "defaultValue" in item) {
      config[item.key] = item.defaultValue;
    }
  }

  return config;
}
