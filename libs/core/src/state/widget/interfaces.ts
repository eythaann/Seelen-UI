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
   * Will auto size the widget to the content size of the element
   * @example
   *  autoSizeByContent: document.body,
   *  autoSizeByContent: document.getElementById("root"),
   * @default undefined
   */
  autoSizeByContent?: HTMLElement | null;
  /**
   * If autoSizeByContent is set, and this is true, will auto size the widget and
   * adjusts the position to fit on screen
   * @default true
   */
  autoSizeFitOnScreen?: boolean;
  /**
   * Will normalize the device pixel ratio to 1:1
   * @default false
   */
  normalizeDevicePixelRatio?: boolean;
  /**
   * Will save the position and size of the widget on change.
   * This is intedeed to be used when the size and position of the widget is
   * allowed to be changed by the user, Normally used on desktop widgets.
   *
   * @default widget.preset === "Desktop"
   */
  saveAndRestoreLastRect?: boolean;
  /**
   * Will hide the widget when the focus is lost.
   *
   * @default widget.preset === "Popup"
   */
  hideOnFocusLoss?: boolean;
  /**
   * Will close the widget when it is hidden after a certain amount of time, instead of just hiding it.
   * This is useful for popup widgets that are not used frequently, as it will free up system resources.
   * The widget will be recreated when it is triggered again.
   *
   * @default widget.lazy === true
   */
  closeOnHide?: boolean;
  /**
   * Will disable the css animations on the widget when performace mode is set to Extreme
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
