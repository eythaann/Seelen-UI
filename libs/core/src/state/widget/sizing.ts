import type { Widget } from "./mod.ts";
import { Alignment } from "@seelen-ui/types";
import { Mutex } from "../../utils/async.ts";
import { fitIntoMonitor } from "./positioning.ts";

class OptimisiticFrame {
  x: number = 0;
  y: number = 0;
  width: number = 0;
  height: number = 0;

  async init(widget: Widget): Promise<void> {
    const { width, height } = await widget.window.outerSize();
    const { x, y } = await widget.window.outerPosition();

    this.width = width;
    this.height = height;
    this.x = x;
    this.y = y;

    widget.window.onResized((e) => {
      OPTIMISTIC_FRAME.runExclusive((ref) => {
        ref.width = e.payload.width;
        ref.height = e.payload.height;
      });
    });

    widget.window.onMoved((e) => {
      OPTIMISTIC_FRAME.runExclusive((ref) => {
        ref.x = e.payload.x;
        ref.y = e.payload.y;
      });
    });
  }
}

export const OPTIMISTIC_FRAME = new Mutex(new OptimisiticFrame());

export class WidgetAutoSizer {
  /** From which side the widget will grow */
  originX?: Alignment | null;
  /** From which side the widget will grow */
  originY?: Alignment | null;

  constructor(
    private widget: Widget,
    private element: HTMLElement,
    private fitOnScreen: boolean,
  ) {
    this.execute = this.execute.bind(this);
    this.setup();
  }

  private setup(): () => void {
    // Disable resizing by the user
    this.widget.window.setResizable(false);

    this.widget.onTrigger(({ alignX, alignY }) => {
      OPTIMISTIC_FRAME.runExclusive(() => {
        this.originX = alignX;
        this.originY = alignY;
      });
    });

    const observer = new ResizeObserver(this.execute);
    observer.observe(this.element, {
      box: "border-box",
    });

    return () => {
      observer.disconnect();
    };
  }

  async execute(): Promise<void> {
    const guard = await OPTIMISTIC_FRAME.acquire();
    const { x, y, width, height } = guard.value;

    let frame = {
      x,
      y,
      width: Math.ceil(this.element.scrollWidth * globalThis.window.devicePixelRatio),
      height: Math.ceil(this.element.scrollHeight * globalThis.window.devicePixelRatio),
    };

    const widthDiff = frame.width - width;
    const heightDiff = frame.height - height;

    // Only update if the difference is more than 1px (avoid infinite loops from decimal differences)
    if (widthDiff === 0 && heightDiff === 0) {
      guard.release();
      return;
    }

    /* console.debug(
      `Auto resizing from ${width}x${height} to ${frame.width}x${frame.height} using ${this.originX}/${this.originY} origin`,
    ); */

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

    if (this.fitOnScreen) {
      frame = fitIntoMonitor(frame);
    }

    try {
      await this.widget.__unsafe_setPosition(
        {
          left: frame.x,
          top: frame.y,
          right: frame.x + frame.width,
          bottom: frame.y + frame.height,
        },
        guard.value,
      );
    } finally {
      guard.release();
    }
  }
}
