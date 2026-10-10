import {
  type Alignment,
  type GenericWidgetSettings,
  type Rect,
  type Widget as IWidget,
  type WidgetConfigDefinition,
  type WidgetId,
  WidgetPreset,
  type WidgetSettingItem,
  WidgetStatus,
} from "@seelen-ui/types";
import { invoke, SeelenCommand, SeelenEvent } from "../../handlers/mod.ts";
import { decodeBase64Url } from "@std/encoding";
import { debounce } from "../../utils/async.ts";
import { adjustPositionByPlacement, fitIntoMonitor, initMonitorsState } from "./positioning.ts";
import { startThemingTool } from "../theme/theming.ts";
import type { InitWidgetOptions, ReadyWidgetOptions, WidgetInformation } from "./interfaces.ts";
import { disableAnimationsOnPerformanceMode } from "./performance.ts";
import { subscribe } from "../../handlers/mod.ts";
import { WidgetBasics } from "./abstractions/mod.ts";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { OPTIMISTIC_FRAME } from "./abstractions/3_autosize.ts";

interface WidgetInternalState {
  initialized: boolean;
  ready: boolean;
}

/**
 * Represents the widget instance running in the current webview
 */
export class Widget extends WidgetBasics {
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

  private runtimeState: WidgetInternalState = {
    initialized: false,
    ready: false,
  };

  private constructor(widget: IWidget) {
    super();

    this.def = widget;

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

  /** Returns if the widget is ready */
  get isReady(): boolean {
    return this.runtimeState.ready;
  }

  /** Returns the default config of the widget, declared on the widget definition */
  public getDefaultConfig(): GenericWidgetSettings {
    const config: GenericWidgetSettings = { enabled: true };
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
      // After positioning, re-run the autosizer to correct any size discrepancy. The trigger
      // reads OPTIMISTIC_FRAME which may be stale: onResized events from prior async resizes
      // can arrive before the trigger fires and roll back the optimistic size to an intermediate
      // value. If the trigger then repositions with that stale size its SetSelfPosition call
      // lands in the Win32 queue after the correct resize, overwriting it. Running execute()
      // here detects the remaining diff and issues a corrective resize + position adjustment.
      if (this.autoSize.enabled) {
        await this.executeAutoSize();
      }
    });
  }

  private hideOnFocusLoss(): void {
    let wasFocused = false;

    const hideDelayed = debounce(() => {
      this.hide();
    }, 100);

    subscribe(SeelenEvent.GlobalFocusChanged, ({ payload: focused }) => {
      if (focused.hwnd !== this.windowId && focused.ownerHwnd !== this.windowId) {
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

    if (x && y) {
      const frame = await OPTIMISTIC_FRAME.runExclusive((ref) => ({
        x: Number(x),
        y: Number(y),
        width: this.autoSize.enabled ? ref.width : Number(width),
        height: this.autoSize.enabled ? ref.height : Number(height),
      }));

      const safeFrame = fitIntoMonitor(frame);
      await this.setPosition({
        left: safeFrame.x,
        top: safeFrame.y,
        right: safeFrame.x + safeFrame.width,
        bottom: safeFrame.y + safeFrame.height,
      });
    }

    this.onMoved(
      debounce((e) => {
        const { x, y } = e.payload;
        storage.setItem(`x`, x.toString());
        storage.setItem(`y`, y.toString());
        console.info(`Widget position saved: ${x} ${y}`);
      }, 500),
    );

    if (!this.autoSize.enabled) {
      this.onResized(
        debounce((e) => {
          const { width, height } = e.payload;
          storage.setItem(`width`, width.toString());
          storage.setItem(`height`, height.toString());
          console.info(`Widget size saved: ${width} ${height}`);
        }, 500),
      );
    }
  }

  /**
   * Makes the webview render 1 CSS px = 1 physical px on every monitor.
   *
   * Done natively by pinning the WebView2 rasterization scale to 1.0 and disabling its
   * own monitor DPI tracking. A zoom-based compensation from here (`zoom = 1 / dpr`) is
   * unreliable on multi-monitor/mixed-DPI setups: WebView2 updates its rasterization scale
   * asynchronously after a monitor change and `devicePixelRatio` lags behind `setZoom`,
   * so the correction gets computed from stale readings and overshoots.
   */
  private async normalizeDevicePixelRatio(): Promise<void> {
    await invoke(SeelenCommand.NormalizeSelfDevicePixelRatio);
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
    await this.prepare();

    this._destroyOnHide = options.closeOnHide ?? this.def.lazy;

    if (options.normalizeDevicePixelRatio) {
      await this.normalizeDevicePixelRatio();
    }

    await initMonitorsState();
    await OPTIMISTIC_FRAME.runExclusive((state) => {
      state.init(this);
    });

    if (options.autoSizeByContent) {
      this.setupAutoSizer(options.autoSizeByContent, options.autoSizeFitOnScreen ?? true);
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

    if (options.useThemes ?? true) {
      await startThemingTool();
    }

    if (options.disableCssAnimations ?? true) {
      await disableAnimationsOnPerformanceMode();
    } else {
      console.trace("Animations won't be disabled because widget configuration");
    }
  }

  /** Tasks to be executed before the widget be marked as ready */
  public preReadyTasks: Promise<void>[] = [];

  /**
   * Will mark the widget as `ready` and pool pending triggers.
   *
   * If the widget is not lazy this will inmediately show the widget.
   * Lazy widget should be shown on trigger action.
   */
  public async ready(options: ReadyWidgetOptions = {}): Promise<void> {
    if (!this.runtimeState.initialized) {
      throw new Error(`Widget was not initialized before ready`);
    }

    if (this.runtimeState.ready) {
      console.warn(`Widget is already ready`);
      return;
    }
    this.runtimeState.ready = true;

    const { show = !this.def.lazy } = options;

    const alreadyVisible = await this.window.isVisible();
    globalThis.document.documentElement.toggleAttribute("data-widget-hidden", !alreadyVisible);
    // pre-compute styles
    globalThis.getComputedStyle(globalThis.document.documentElement).opacity;

    if (this.autoSize.enabled) {
      await this.executeAutoSize();
    }

    for (const task of this.preReadyTasks) {
      await task;
    }

    globalThis.document.documentElement.dataset.widgetReady = "";
    // this will mark the widget as ready, and send pending trigger event if exists
    await invoke(SeelenCommand.SetCurrentWidgetStatus, { status: WidgetStatus.Ready });

    if (show && !alreadyVisible) {
      await this.show();
    }
  }

  private _attach: { enabled: boolean; unsub?: () => void; rect?: Rect } = { enabled: false };
  /**
   * If for some reason the widget position is changed (like caused by system on system bars addition)
   * this will reposition the widget to the last declared rectangle.
   *
   * Caution: call this only if you are sure not other parts move/resize the widget or will cause flickering.
   */
  public attachPosition(): void {
    if (this._attach.enabled || this.autoSize.enabled) {
      return;
    }

    this._attach.enabled = true;
    this._attach.unsub = this.onRectChange((actual) => {
      if (!this._attach.rect) return;
      const old = this._attach.rect;
      if (
        old.left !== actual.left ||
        old.top !== actual.top ||
        old.right !== actual.right ||
        old.bottom !== actual.bottom
      ) {
        this.setPosition(old!);
      }
    });
  }

  public unattachPosition(): void {
    this._attach.enabled = false;
    this._attach.unsub?.();
    this._attach.unsub = undefined;
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

      const newRect = {
        left: adjusted.x,
        top: adjusted.y,
        right: adjusted.x + adjusted.width,
        bottom: adjusted.y + adjusted.height,
      };
      if (this._attach.enabled) {
        this._attach.rect = { ...newRect };
      }
      await Widget.self.__unsafe_setSelfPosition(newRect, ref);
    });
  }

  public async setPosition(rect: Rect): Promise<void> {
    if (this._attach.enabled) {
      this._attach.rect = { ...rect };
    }
    await OPTIMISTIC_FRAME.runExclusive(async (frame) => {
      await this.__unsafe_setSelfPosition(rect, frame);
    });
  }

  /**
   * Will force foreground the widget.\
   * No need to focus the webview itself, wry moves the focus into it on the window `WM_SETFOCUS`.
   */
  public async focus(): Promise<void> {
    await invoke(SeelenCommand.RequestFocus, { hwnd: this.windowId }).catch(() => {});
  }
}

type ExtendedGlobalThis = typeof globalThis & {
  __SLU_WIDGET?: IWidget;
  __SLU_WIDGET_INSTANCE?: Widget;
};

export const SeelenSettingsWidgetId: WidgetId = "@seelen/settings" as WidgetId;
export const SeelenPopupWidgetId: WidgetId = "@seelen/dialog" as WidgetId;
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
