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
   * Automatically resizes the widget to match the content size of a given element.
   *
   * @example
   *  autoSizeByContent: document.body
   *  autoSizeByContent: document.getElementById("root")
   *
   * @default undefined
   */
  autoSizeByContent?: HTMLElement | null;

  /**
   * When `autoSizeByContent` is enabled, ensures the widget stays fully visible
   * by adjusting its position to fit within the screen bounds.
   *
   * @default true
   */
  autoSizeFitOnScreen?: boolean;

  /**
   * Forces the WebView to use a 1:1 device pixel ratio (disables DPI scaling).
   *
   * ⚠️ Not intended for typical widgets. Use only when exact pixel alignment
   * with the physical display is required (e.g. multi-monitor widgets like
   * power menu, workspace viewer, wallpaper manager).
   *
   * Widgets using this must handle per-monitor DPI scaling manually.
   *
   * @default false
   */
  normalizeDevicePixelRatio?: boolean;

  /**
   * Persists and restores the widget's last known position and size.
   *
   * Intended for widgets that users can move or resize (e.g. desktop widgets).
   *
   * @default widget.preset === "Desktop"
   */
  saveAndRestoreLastRect?: boolean;

  /**
   * Hides the widget when it loses focus.
   *
   * Commonly used for popup-style widgets.
   *
   * @default widget.preset === "Popup"
   */
  hideOnFocusLoss?: boolean;

  /**
   * Closes the widget after being hidden for a period of time,
   * instead of keeping it in memory.
   *
   * Useful for infrequently used popup widgets to reduce resource usage.
   * The widget will be recreated when opened again.
   *
   * @default widget.lazy === true
   */
  closeOnHide?: boolean;

  /**
   * Disables CSS animations when performance mode is set to "Extreme".
   *
   * Helps reduce rendering overhead on low-end systems or heavy workloads.
   *
   * @default true
   */
  disableCssAnimations?: boolean;
}

export interface ReadyWidgetOptions {
  /**
   * If show the widget on Ready
   *
   * @default !widget.lazy
   */
  show?: boolean;
}
