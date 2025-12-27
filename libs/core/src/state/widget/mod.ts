import {
  type ThirdPartyWidgetSettings,
  type Widget as IWidget,
  type WidgetId,
  WidgetPreset,
  WidgetStatus,
  type WidgetTriggerPayload,
  type WsdGroupEntry,
} from "@seelen-ui/types";
import { invoke, SeelenCommand, SeelenEvent, type UnSubscriber } from "../../handlers/mod.ts";
import { List } from "../../utils/List.ts";
import { newFromInvoke, newOnEvent } from "../../utils/State.ts";
import { getCurrentWebviewWindow, type WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { decodeBase64Url } from "@std/encoding";
import { PhysicalPosition, PhysicalSize } from "@tauri-apps/api/dpi";
import { monitorFromPoint } from "@tauri-apps/api/window";
import { debounce } from "../../utils/async.ts";
import { WidgetAutoSizer } from "./sizing.ts";
import { adjustPostionByPlacement } from "./positioning.ts";
import { startThemingTool } from "../theme/theming.ts";

export const SeelenSettingsWidgetId: WidgetId = "@seelen/settings" as WidgetId;
export const SeelenPopupWidgetId: WidgetId = "@seelen/popup" as WidgetId;
export const SeelenWegWidgetId: WidgetId = "@seelen/weg" as WidgetId;
export const SeelenToolbarWidgetId: WidgetId = "@seelen/fancy-toolbar" as WidgetId;
export const SeelenWindowManagerWidgetId: WidgetId = "@seelen/window-manager" as WidgetId;
export const SeelenWallWidgetId: WidgetId = "@seelen/wallpaper-manager" as WidgetId;

export class WidgetList extends List<IWidget> {
  static getAsync(): Promise<WidgetList> {
    return newFromInvoke(this, SeelenCommand.StateGetWidgets);
  }

  static onChange(cb: (payload: WidgetList) => void): Promise<UnSubscriber> {
    return newOnEvent(cb, this, SeelenEvent.StateWidgetsChanged);
  }

  findById(id: WidgetId): IWidget | undefined {
    return this.asArray().find((widget) => widget.id === id);
  }
}

export interface WidgetInformation {
  /** decoded webview label */
  label: string;
  /** Will be present if the widget replicas is set to by monitor */
  monitorId: string | null;
  /** Will be present if the widget replicas is set to multiple */
  instanceId: string | null;
  /** params present on the webview label */
  params: { readonly [key in string]?: string };
}

export interface InitWidgetOptions {
  /**
   * If show the widget on Ready
   *
   * @default !widget.lazy
   */
  show?: boolean;
  /**
   * Will auto size the widget to the content size of the element
   * @example
   *  autoSizeByContent: document.body,
   *  autoSizeByContent: document.getElementById("root"),
   * @default undefined
   */
  autoSizeByContent?: HTMLElement | null;
  /**
   * Will save the position and size of the widget on change.
   * This is intedeed to be used when the size and position of the widget is
   * allowed to be changed by the user, Normally used on desktop widgets.
   *
   * @default widget.preset === "Desktop"
   */
  saveAndRestoreLastRect?: boolean;
}

interface WidgetInternalState {
  initialized: boolean;
  ready: boolean;
}

/**
 * Represents the widget instance running in the current webview
 */
export class Widget {
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
    initialized: false,
    ready: false,
  };

  private constructor(widget: IWidget) {
    this.def = widget;
    this.webview = getCurrentWebviewWindow();

    const [id, query] = Widget.getDecodedWebviewLabel();
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

  private static getDecodedWebviewLabel(): [WidgetId, string | undefined] {
    const encondedLabel = getCurrentWebviewWindow().label;
    const decodedLabel = new TextDecoder().decode(decodeBase64Url(encondedLabel));
    const [id, query] = decodedLabel.split("?");
    if (!id) {
      throw new Error("Missing widget id on webview label");
    }
    return [id as WidgetId, query];
  }

  /** Will throw if the library is being used on a non Seelen UI environment */
  static getCurrentWidgetId(): WidgetId {
    return this.getCurrent().id;
  }

  /** Will throw if the library is being used on a non Seelen UI environment */
  static getCurrent(): Widget {
    const scope = globalThis as ExtendedGlobalThis;
    if (!scope.__SLU_WIDGET) {
      throw new Error("The library is being used on a non Seelen UI environment");
    }
    return (
      scope.__SLU_WIDGET_INSTANCE || (scope.__SLU_WIDGET_INSTANCE = new Widget(scope.__SLU_WIDGET))
    );
  }

  private static getEntryDefaultValues(entry: WsdGroupEntry): Record<string, unknown> {
    const config: Record<string, unknown> = {
      [entry.config.key]: entry.config.defaultValue,
    };
    for (const item of entry.children) {
      Object.assign(config, Widget.getEntryDefaultValues(item));
    }
    return config;
  }

  /** Returns the default config of the widget, declared on the widget definition */
  public getDefaultConfig(): ThirdPartyWidgetSettings {
    const config: ThirdPartyWidgetSettings = { enabled: true };
    for (const { group } of this.def.settings) {
      for (const entry of group) {
        Object.assign(config, Widget.getEntryDefaultValues(entry));
      }
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
    await Promise.all([
      ...this.applyInvisiblePreset(),
      // Desktop widgets are always on bottom
      this.webview.setAlwaysOnBottom(true),
    ]);
  }

  /** Will apply the recommended settings for an overlay widget */
  private async applyOverlayPreset(): Promise<void> {
    await Promise.all([
      ...this.applyInvisiblePreset(),
      // Overlay widgets are always on top
      this.webview.setAlwaysOnTop(true),
    ]);
  }

  /** Will apply the recommended settings for a popup widget */
  private async applyPopupPreset(): Promise<void> {
    await Promise.all([...this.applyInvisiblePreset()]);

    // auto close after 1 minute when not in use to save resources
    const closeOnTimeout = debounce(() => {
      this.webview.close();
    }, 60_000);

    const hideWebview = debounce(() => {
      this.webview.hide();
      closeOnTimeout();
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

      if (desiredPosition) {
        const { width, height } = await this.webview.outerSize();
        const pos = await adjustPostionByPlacement({
          x: desiredPosition[0],
          y: desiredPosition[1],
          width,
          height,
          alignX,
          alignY,
        });
        await this.webview.setPosition(new PhysicalPosition(pos.x, pos.y));
      }
      await this.webview.show();
      await this.webview.setFocus();
    });
  }

  /**
   * Will restore the saved position and size of the widget on start,
   * after that will store the position and size of the widget on change.
   */
  public async persistPositionAndSize(): Promise<void> {
    const storage = globalThis.window.localStorage;
    const { label } = this.webview;

    const [x, y, width, height] = [`x`, `y`, `width`, `height`].map((k) => storage.getItem(`${label}::${k}`));

    if (x && y) {
      const pos = new PhysicalPosition(Number(x), Number(y));
      // check if the stored position is still valid
      const monitor = await monitorFromPoint(pos.x, pos.y);
      if (monitor) {
        await this.webview.setPosition(pos);
      }
    }

    if (width && height) {
      const size = new PhysicalSize(Number(width), Number(height));
      await this.webview.setSize(size);
    }

    this.webview.onMoved(
      debounce((e) => {
        const { x, y } = e.payload;
        storage.setItem(`${label}::x`, x.toString());
        storage.setItem(`${label}::y`, y.toString());
        console.info(`Widget position saved: ${x} ${y}`);
      }, 500),
    );

    this.webview.onResized(
      debounce((e) => {
        const { width, height } = e.payload;
        storage.setItem(`${label}::width`, width.toString());
        storage.setItem(`${label}::height`, height.toString());
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

    this.runtimeState.initialized = true;
    this.initOptions = options;

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

    if (options.autoSizeByContent) {
      this.autoSizer = new WidgetAutoSizer(this.webview, options.autoSizeByContent);
    } else if (options.saveAndRestoreLastRect ?? this.def.preset === WidgetPreset.Desktop) {
      await this.persistPositionAndSize();
    }
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
      await this.webview.show();
    }

    // this will mark the widget as ready, and send pending trigger event if exists
    await invoke(SeelenCommand.SetCurrentWidgetStatus, { status: WidgetStatus.Ready });
  }

  public onTrigger(cb: (args: WidgetTriggerPayload) => void): void {
    this.webview.listen<WidgetTriggerPayload>(SeelenEvent.WidgetTriggered, ({ payload }) => {
      // fix for cutted popups, ensure correct size on trigger.
      // await this.autoSizer?.execute();
      cb(payload);
    });
  }
}

type ExtendedGlobalThis = typeof globalThis & {
  __SLU_WIDGET?: IWidget;
  __SLU_WIDGET_INSTANCE?: Widget;
};
