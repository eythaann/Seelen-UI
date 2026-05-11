import {
  type Alignment,
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
import { decodeBase64Url } from "@std/encoding";
import { debounce } from "../../utils/async.ts";
import { OPTIMISTIC_FRAME, WidgetAutoSizer } from "./sizing.ts";
import { adjustPositionByPlacement, fitIntoMonitor, initMonitorsState } from "./positioning.ts";
import { startThemingTool } from "../theme/theming.ts";
import type { InitWidgetOptions, ReadyWidgetOptions, WidgetInformation } from "./interfaces.ts";
import { disableAnimationsOnPerformanceMode } from "./performance.ts";
import { getCurrentWebview, type Webview } from "@tauri-apps/api/webview";
import { getCurrentWindow, type Window } from "@tauri-apps/api/window";
import { subscribe } from "../../../mod.ts";

interface WidgetInternalState {
  hwnd: number;
  initialized: boolean;
  ready: boolean;
  firstFocus: boolean;
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
  /** current webview where the widget is running */
  public readonly webview: Webview;
  /** current window where the widget is running */
  public readonly window: Window;

  private autoSizer?: WidgetAutoSizer;
  private destroyOnHide = false;
  private runtimeState: WidgetInternalState = {
    hwnd: 0,
    initialized: false,
    ready: false,
    firstFocus: true,
  };

  private constructor(widget: IWidget) {
    this.def = widget;
    this.webview = getCurrentWebview();
    this.window = getCurrentWindow();

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

  /** Returns the default config of the widget, declared on the widget definition */
  public getDefaultConfig(): ThirdPartyWidgetSettings {
    const config: ThirdPartyWidgetSettings = { enabled: true };
    for (const definition of this.def.settings) {
      Object.assign(config, getDefinitionDefaultValues(definition));
    }
    return config;
  }

  /** Will apply the recommended settings for a desktop widget */
  private applyDesktopPreset(): void {}

  /** Will apply the recommended settings for an overlay widget */
  private applyOverlayPreset(): void {}

  /** Will apply the recommended settings for a popup widget */
  private applyPopupPreset(): void {
    this.onTrigger(async ({ desiredPosition, alignX, alignY }) => {
      if (desiredPosition) {
        await this.adjustAndSetPosition(desiredPosition.x, desiredPosition.y, alignX, alignY);
      }
      await this.show();
      await this.focus();
    });
  }

  private hideOnFocusLoss(): void {
    let wasFocused = false;

    const hideDelayed = debounce(() => {
      this.hide();
    }, 100);

    subscribe(SeelenEvent.GlobalFocusChanged, ({ payload: focused }) => {
      if (focused.hwnd !== this.runtimeState.hwnd && focused.ownerHwnd !== this.runtimeState.hwnd) {
        if (wasFocused) {
          hideDelayed();
        }
        wasFocused = false;
        return;
      }

      wasFocused = true;
      hideDelayed.cancel();
    });
  }

  /**
   * Will restore the saved position and size of the widget on start,
   * after that will store the position and size of the widget on change.
   */
  private async persistPositionAndSize(): Promise<void> {
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

    this.window.onMoved(
      debounce((e) => {
        const { x, y } = e.payload;
        storage.setItem(`x`, x.toString());
        storage.setItem(`y`, y.toString());
        console.info(`Widget position saved: ${x} ${y}`);
      }, 500),
    );

    this.window.onResized(
      debounce((e) => {
        const { width, height } = e.payload;
        storage.setItem(`width`, width.toString());
        storage.setItem(`height`, height.toString());
        console.info(`Widget size saved: ${width} ${height}`);
      }, 500),
    );
  }

  private async normalizeDevicePixelRatio(): Promise<void> {
    // play with zoom level to reset device pixel ratio to 1:1
    let oldDPR = globalThis.devicePixelRatio;
    await this.webview.setZoom(1 / oldDPR);
    this.window.onScaleChanged(() => {
      if (globalThis.devicePixelRatio !== oldDPR) {
        // when zoom was set dpr changed, so in case of change this is accomulative unit
        oldDPR = oldDPR * globalThis.devicePixelRatio;
        this.webview.setZoom(1 / (oldDPR * globalThis.devicePixelRatio));
      }
    });
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
    this.destroyOnHide = options.closeOnHide ?? this.def.lazy;

    if (options.normalizeDevicePixelRatio) {
      await this.normalizeDevicePixelRatio();
    }

    if (options.autoSizeByContent) {
      this.autoSizer = new WidgetAutoSizer(
        this,
        options.autoSizeByContent,
        options.autoSizeFitOnScreen ?? true,
      );
    }

    if (options.saveAndRestoreLastRect ?? this.def.preset === WidgetPreset.Desktop) {
      await this.persistPositionAndSize();
    }

    if (options.hideOnFocusLoss ?? this.def.preset === WidgetPreset.Popup) {
      this.hideOnFocusLoss();
    }

    switch (this.def.preset) {
      case WidgetPreset.None:
        break;
      case WidgetPreset.Desktop:
        this.applyDesktopPreset();
        break;
      case WidgetPreset.Overlay:
        this.applyOverlayPreset();
        break;
      case WidgetPreset.Popup:
        this.applyPopupPreset();
        break;
    }

    await startThemingTool();
    await initMonitorsState();

    if (options.disableCssAnimations ?? true) {
      await disableAnimationsOnPerformanceMode();
    } else {
      console.trace("Animations won't be disabled because widget configuration");
    }

    await OPTIMISTIC_FRAME.runExclusive(async (state) => {
      await state.init(this);
    });
  }

  /**
   * Will mark the widget as `ready` and pool pending triggers.
   *
   * If the widget is not lazy this will inmediately show the widget.
   * Lazy widget should be shown on trigger action.
   */
  public async ready(options: ReadyWidgetOptions = {}): Promise<void> {
    const { show = !this.def.lazy } = options;

    if (!this.runtimeState.initialized) {
      throw new Error(`Widget was not initialized before ready`);
    }

    if (this.runtimeState.ready) {
      console.warn(`Widget is already ready`);
      return;
    }

    this.runtimeState.ready = true;
    await this.autoSizer?.execute();

    if (show && !(await this.window.isVisible())) {
      await this.show();
      // await this.focus();
    }

    // this will mark the widget as ready, and send pending trigger event if exists
    await invoke(SeelenCommand.SetCurrentWidgetStatus, { status: WidgetStatus.Ready });
  }

  public onTrigger(cb: (args: WidgetTriggerPayload) => void): void {
    this.webview.listen<WidgetTriggerPayload>(SeelenEvent.WidgetTriggered, ({ payload }) => {
      cb(payload);
    });
  }

  public async __unsafe_setPosition(rect: Rect, ref: Frame): Promise<void> {
    await invoke(SeelenCommand.SetSelfPosition, {
      rect: {
        left: Math.round(rect.left),
        top: Math.round(rect.top),
        right: Math.round(rect.right),
        bottom: Math.round(rect.bottom),
      },
    });

    // optimistically update state, as arrived event after change is async
    ref.x = rect.left;
    ref.y = rect.top;
    ref.width = rect.right - rect.left;
    ref.height = rect.bottom - rect.top;
  }

  /**
   * This will adjust the position of the widget based on the current placement and alignX/alignY arguments.
   * This makes the widget fit into the monitor where it was placed, avoiding monitor overflow.
   */
  public async adjustAndSetPosition(
    x: number,
    y: number,
    alignX?: Alignment | null,
    alignY?: Alignment | null,
  ): Promise<void> {
    await OPTIMISTIC_FRAME.runExclusive(async (ref) => {
      const adjusted = adjustPositionByPlacement({
        frame: {
          x,
          y,
          width: ref.width,
          height: ref.height,
        },
        originX: alignX,
        originY: alignY,
      });

      await Widget.self.__unsafe_setPosition(
        {
          left: adjusted.x,
          top: adjusted.y,
          right: adjusted.x + adjusted.width,
          bottom: adjusted.y + adjusted.height,
        },
        ref,
      );
    });
  }

  public async setPosition(rect: Rect): Promise<void> {
    await OPTIMISTIC_FRAME.runExclusive(async (frame) => {
      await this.__unsafe_setPosition(rect, frame);
    });
  }

  public async show(): Promise<void> {
    debouncedClose.cancel();
    await this.window.show();
  }

  /** Will force foreground the widget */
  public async focus(): Promise<void> {
    if (this.runtimeState.firstFocus) {
      await getCurrentWebview().setFocus();
      this.runtimeState.firstFocus = false;
    }
    await invoke(SeelenCommand.RequestFocus, { hwnd: this.runtimeState.hwnd }).catch(() => {});
  }

  public hide(): void {
    this.window.hide();
    if (this.destroyOnHide) {
      debouncedClose();
    }
  }
}

const debouncedClose = debounce(() => {
  Widget.self.window.close();
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
  const encondedLabel = getCurrentWebview().label;
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
