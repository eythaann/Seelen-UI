import type { ThirdPartyWidgetSettings, Widget as IWidget, WidgetId, WsdGroupEntry } from "@seelen-ui/types";
import { SeelenCommand, SeelenEvent, type UnSubscriber } from "../../handlers/mod.ts";
import { List } from "../../utils/List.ts";
import { newFromInvoke, newOnEvent } from "../../utils/State.ts";
import { getCurrentWebviewWindow, type WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { decodeBase64Url } from "@std/encoding/base64url";
import { PhysicalPosition, PhysicalSize } from "@tauri-apps/api/dpi";
import { monitorFromPoint } from "@tauri-apps/api/window";
import { debounce } from "../../utils/async.ts";

export const SeelenSettingsWidgetId: WidgetId = "@seelen/settings" as WidgetId;
export const SeelenPopupWidgetId: WidgetId = "@seelen/popup" as WidgetId;
export const SeelenWegWidgetId: WidgetId = "@seelen/weg" as WidgetId;
export const SeelenToolbarWidgetId: WidgetId = "@seelen/fancy-toolbar" as WidgetId;
export const SeelenWindowManagerWidgetId: WidgetId = "@seelen/window-manager" as WidgetId;
export const SeelenLauncherWidgetId: WidgetId = "@seelen/launcher" as WidgetId;
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

interface WidgetInformation {
  /** decoded webview label */
  label: string;
  /** Will be present if the widget replicas is set to by monitor */
  monitorId: string | null;
  /** Will be present if the widget replicas is set to multiple */
  instanceId: string | null;
  /** params present on the webview label */
  params: { readonly [key in string]?: string };
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
    const decodedLabel = new TextDecoder().decode(
      decodeBase64Url(encondedLabel),
    );
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
      throw new Error(
        "The library is being used on a non Seelen UI environment",
      );
    }
    return scope.__SLU_WIDGET_INSTANCE ||
      (scope.__SLU_WIDGET_INSTANCE = new Widget(scope.__SLU_WIDGET));
  }

  /** @deprecated Use `getCurrent` instead, this method will be removed on future */
  private static getCurrentAsync(): Widget {
    return this.getCurrent();
  }

  private static getEntryDefaultValues(
    entry: WsdGroupEntry,
  ): Record<string, unknown> {
    const config: Record<string, unknown> = {
      [entry.config.key]: entry.config.defaultValue,
    };
    for (const item of entry.children) {
      Object.assign(config, Widget.getEntryDefaultValues(item));
    }
    return config;
  }

  /** Returns the default config of the widget, declared on the widget definition */
  getDefaultConfig(): ThirdPartyWidgetSettings {
    const config: ThirdPartyWidgetSettings = { enabled: true };
    for (const { group } of this.def.settings) {
      for (const entry of group) {
        Object.assign(config, Widget.getEntryDefaultValues(entry));
      }
    }
    return config;
  }

  private commonConfig(): Array<Promise<void>> {
    return [
      this.webview.setDecorations(false), // no title bar
      this.webview.setShadow(false), // no shadows
      // hide from native shell
      this.webview.setSkipTaskbar(true),
      // set as hidden (no manage) window on seelen ui
      this.webview.title().then((title) => {
        this.webview.setTitle(`.${title}`);
      }),
      // as a (desktop/overlay) widget we don't wanna allow nothing of these
      this.webview.setMinimizable(false),
      this.webview.setMaximizable(false),
      this.webview.setClosable(false),
    ];
  }

  /** Will set this instance as a desktop widget */
  async setAsDesktopWidget(): Promise<void> {
    await Promise.all([
      ...this.commonConfig(),
      // Desktop widgets are always on bottom
      this.webview.setAlwaysOnBottom(true),
    ]);
  }

  /** Will set this instance as an overlay widget */
  async setAsOverlayWidget(): Promise<void> {
    await Promise.all([
      ...this.commonConfig(),
      // Overlay widgets are always on top
      this.webview.setAlwaysOnTop(true),
    ]);
  }

  /**
   * Will restore the saved position and size of the widget after that
   * will store the position and size of the widget on change.
   */
  async persistPositionAndSize(): Promise<void> {
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

    this.webview.onMoved(debounce((e) => {
      const { x, y } = e.payload;
      storage.setItem(`${label}::x`, x.toString());
      storage.setItem(`${label}::y`, y.toString());
      console.info(`Widget position saved: ${x} ${y}`);
    }, 500));

    this.webview.onResized(debounce((e) => {
      const { width, height } = e.payload;
      storage.setItem(`${label}::width`, width.toString());
      storage.setItem(`${label}::height`, height.toString());
      console.info(`Widget size saved: ${width} ${height}`);
    }, 500));
  }
}

type ExtendedGlobalThis = typeof globalThis & {
  __SLU_WIDGET?: IWidget;
  __SLU_WIDGET_INSTANCE?: Widget;
};
