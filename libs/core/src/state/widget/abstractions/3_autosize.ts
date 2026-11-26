import { Alignment } from "@seelen-ui/types";
import { Mutex } from "../../../utils/async.ts";
import { fitIntoMonitor } from "../positioning.ts";
import { Widget_2 } from "./2_triggering.ts";

export class OptimisiticFrame {
  x: number = 0;
  y: number = 0;
  width: number = 0;
  height: number = 0;

  init(widget: Widget_2): void {
    const { x, y, width, height } = widget.frame;

    this.width = width;
    this.height = height;
    this.x = x;
    this.y = y;

    widget.onResized((e) => {
      OPTIMISTIC_FRAME.runExclusive((ref) => {
        ref.width = e.width;
        ref.height = e.height;
      });
    });

    widget.onMoved((e) => {
      OPTIMISTIC_FRAME.runExclusive((ref) => {
        ref.x = e.x;
        ref.y = e.y;
      });
    });
  }
}

export const OPTIMISTIC_FRAME = new Mutex(new OptimisiticFrame());

interface AutoSizerState {
  enabled: boolean;
  /** From which side the widget will grow */
  originX?: Alignment | null;
  /** From which side the widget will grow */
  originY?: Alignment | null;
  element: HTMLElement;
  fitOnScreen: boolean;
}

export class Widget_3 extends Widget_2 {
  private autoSizeFrame: number | null = null;
  private autoSizePending = false;
  private autoSizeTask: Promise<void> | null = null;

  protected autoSize: AutoSizerState = {
    enabled: false,
    element: document.body,
    fitOnScreen: true,
  };

  constructor() {
    super();
    this.executeAutoSize = this.executeAutoSize.bind(this);
    this.scheduleAutoSize = this.scheduleAutoSize.bind(this);
  }

  protected setupAutoSizer(element: HTMLElement, fitOnScreen: boolean): void {
    this.autoSize = { ...this.autoSize, element, fitOnScreen, enabled: true };

    // Disable resizing by the user
    this.window.setResizable(false);

    this.onTrigger(({ alignX, alignY }) => {
      OPTIMISTIC_FRAME.runExclusive(() => {
        this.autoSize.originX = alignX;
        this.autoSize.originY = alignY;
      });
    });

    const observer = new ResizeObserver(this.scheduleAutoSize);
    observer.observe(element, {
      box: "border-box",
    });
  }

  private scheduleAutoSize(): void {
    if (globalThis.document.visibilityState === "hidden") {
      if (this.autoSizeFrame !== null) globalThis.cancelAnimationFrame(this.autoSizeFrame);
      this.autoSizeFrame = null;
      this.executeAutoSize().catch(console.error);
      return;
    }

    if (this.autoSizeFrame !== null) return;
    this.autoSizeFrame = globalThis.requestAnimationFrame(() => {
      this.autoSizeFrame = null;
      this.executeAutoSize().catch(console.error);
    });
  }

  protected executeAutoSize(): Promise<void> {
    this.autoSizePending = true;
    if (!this.autoSizeTask) {
      this.autoSizeTask = this.flushAutoSize().finally(() => {
        this.autoSizeTask = null;
      });
    }
    return this.autoSizeTask;
  }

  private async flushAutoSize(): Promise<void> {
    while (this.autoSizePending) {
      this.autoSizePending = false;
      await this.applyAutoSize();
    }
  }

  private async applyAutoSize(): Promise<void> {
    const guard = await OPTIMISTIC_FRAME.acquire();
    const { x, y, width, height } = guard.value;

    let frame = {
      x,
      y,
      width: Math.ceil(this.autoSize.element.scrollWidth * globalThis.window.devicePixelRatio),
      height: Math.ceil(this.autoSize.element.scrollHeight * globalThis.window.devicePixelRatio),
    };

    const widthDiff = frame.width - width;
    const heightDiff = frame.height - height;

    // Only update if the difference is more than 1px (avoid infinite loops from decimal differences)
    if (widthDiff === 0 && heightDiff === 0) {
      guard.release();
      return;
    }

    if (this.autoSize.originX === Alignment.Center) {
      frame.x -= widthDiff / 2;
    } else if (this.autoSize.originX === Alignment.End) {
      frame.x -= widthDiff;
    }

    if (this.autoSize.originY === Alignment.Center) {
      frame.y -= heightDiff / 2;
    } else if (this.autoSize.originY === Alignment.End) {
      frame.y -= heightDiff;
    }

    if (this.autoSize.fitOnScreen) {
      frame = fitIntoMonitor(frame);
    }

    try {
      await this.__unsafe_setSelfPosition(
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
