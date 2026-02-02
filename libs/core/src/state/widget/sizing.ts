import type { Widget } from "./mod.ts";
import { Alignment } from "@seelen-ui/types";
import { PhysicalSize } from "@seelen-ui/lib/tauri";

export class WidgetAutoSizer {
  /** From which side the widget will grow */
  originX: Alignment = Alignment.Start;
  /** From which side the widget will grow */
  originY: Alignment = Alignment.Start;

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
    const { x, y, width, height } = this.widget.frame;

    const frame = {
      x,
      y,
      width: Math.ceil(this.element.scrollWidth * globalThis.window.devicePixelRatio),
      height: Math.ceil(this.element.scrollHeight * globalThis.window.devicePixelRatio),
    };

    const widthDiff = frame.width - width;
    const heightDiff = frame.height - height;

    // Only update if the difference is more than 1px (avoid infinite loops from decimal differences)
    if (widthDiff === 0 && heightDiff === 0) {
      return;
    }

    console.trace(`Auto resize from ${width}x${height} to ${frame.width}x${frame.height}`);

    if (this.originX === Alignment.Center) {
      frame.x -= widthDiff / 2;
    } else if (this.originX === Alignment.End) {
      frame.x -= widthDiff;
    }

    if (this.originY === Alignment.Center) {
      frame.y -= heightDiff / 2;
    } else if (this.originY === Alignment.End) {
      frame.y -= heightDiff;
    }

    // only update size no position on this case
    if (frame.x === x && frame.y === y) {
      await this.widget.webview.setSize(new PhysicalSize(frame.width, frame.height));
      return;
    }

    await this.widget.setPosition({
      left: frame.x,
      top: frame.y,
      right: frame.x + frame.width,
      bottom: frame.y + frame.height,
    });
  }
}
