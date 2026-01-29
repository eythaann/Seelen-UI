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
