import type { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { LogicalSize } from "@tauri-apps/api/dpi";

export interface AutoSizeOptions {
  onResize?: () => void;
}

export class WidgetAutoSizer {
  constructor(private webview: WebviewWindow, private element: HTMLElement) {
    this.execute = this.execute.bind(this);
    this.setup();
  }

  private setup(): () => void {
    // Disable resizing by the user
    this.webview.setResizable(false);

    // Update size when content changes
    const observer = new MutationObserver(this.execute);
    observer.observe(this.element, {
      childList: true,
      subtree: true,
      attributes: true,
      characterData: true,
    });

    // Cleanup function
    return () => {
      observer.disconnect();
    };
  }

  async execute(): Promise<void> {
    const contentWidth = this.element.scrollWidth;
    const contentHeight = this.element.scrollHeight;

    if (contentWidth < 1 || contentHeight < 1) {
      return;
    }

    const { width: physicalWidth, height: physicalHeight } = await this.webview.outerSize();
    const logicalWidth = physicalWidth * globalThis.window.devicePixelRatio;
    const logicalHeight = physicalHeight * globalThis.window.devicePixelRatio;

    // Only update if the difference is more than 1px (avoid infinite loops from decimal differences)
    if (Math.abs(contentWidth - logicalWidth) > 1 || Math.abs(contentHeight - logicalHeight) > 1) {
      const size = new LogicalSize(contentWidth, contentHeight);
      await this.webview.setSize(size);
    }
  }
}
