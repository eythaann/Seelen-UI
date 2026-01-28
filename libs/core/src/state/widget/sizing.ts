import { fitIntoMonitor } from "./positioning.ts";
import type { Widget } from "./mod.ts";

export class WidgetAutoSizer {
  constructor(
    private widget: Widget,
    private element: HTMLElement,
  ) {
    this.execute = this.execute.bind(this);
    this.setup();
  }

  private setup(): () => void {
    // Disable resizing by the user
    this.widget.webview.setResizable(false);

    const observer = new ResizeObserver(this.execute);
    observer.observe(this.element, {
      box: "border-box",
    });

    return () => {
      observer.disconnect();
    };
  }

  async execute(): Promise<void> {
    const newWidth = Math.ceil(this.element.scrollWidth * globalThis.window.devicePixelRatio);
    const newHeight = Math.ceil(this.element.scrollHeight * globalThis.window.devicePixelRatio);

    if (newWidth < 1 || newHeight < 1) {
      return;
    }

    const { x, y, width, height } = this.widget.frame;

    // Only update if the difference is more than 1px (avoid infinite loops from decimal differences)
    if (Math.abs(newWidth - width) > 1 || Math.abs(newHeight - height) > 1) {
      console.trace(`Auto resize from ${width}x${height} to ${newWidth}x${newHeight}`);

      const frame = fitIntoMonitor({ x, y, width: newWidth, height: newHeight });
      await this.widget.setPosition({
        left: frame.x,
        top: frame.y,
        right: frame.x + frame.width,
        bottom: frame.y + frame.height,
      });
    }
  }
}
