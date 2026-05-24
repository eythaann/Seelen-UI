import { SeelenEvent } from "@seelen-ui/lib";
import { window as TauriWindow } from "@seelen-ui/lib/tauri";

class LayeredHitbox {
  private _isIgnoringCursorEvents: boolean = true;
  public firstClick: boolean = true;

  get isIgnoringCursorEvents(): boolean {
    return this._isIgnoringCursorEvents;
  }

  set isIgnoringCursorEvents(value: boolean) {
    if (value == false) {
      this.firstClick = true;
    }
    this._isIgnoringCursorEvents = value;
  }
}

export interface LayeredHitboxRect {
  x: number;
  y: number;
  width: number;
  height: number;
}

/**
 * @param shouldAllowMouseEvent - returns true if mouse events should be allowed for the element
 * @param getPhysicalRect - optional getter for the expected physical window rect (physical pixels).
 *   When provided, this is used instead of tracking via onMoved/onResized events, which prevents
 *   stale coordinates during brief position corrections (e.g. AppBar registration adjustments).
 */
export async function declareDocumentAsLayeredHitbox(
  shouldAllowMouseEvent: (element: Element) => boolean = (element) => element != document.body,
  getPhysicalRect?: () => LayeredHitboxRect,
): Promise<() => void> {
  const window = TauriWindow.getCurrentWindow();

  await window.setIgnoreCursorEvents(true);
  const data = new LayeredHitbox();

  let unlistenMoved: (() => void) | undefined;
  let unlistenResized: (() => void) | undefined;
  const trackedRect: LayeredHitboxRect = { x: 0, y: 0, width: 0, height: 0 };

  if (!getPhysicalRect) {
    unlistenMoved = await window.onMoved((e) => {
      trackedRect.x = e.payload.x;
      trackedRect.y = e.payload.y;
    });

    unlistenResized = await window.onResized((e) => {
      trackedRect.width = e.payload.width;
      trackedRect.height = e.payload.height;
    });

    const { x, y } = await window.outerPosition();
    trackedRect.x = x;
    trackedRect.y = y;
    const { width, height } = await window.outerSize();
    trackedRect.width = width;
    trackedRect.height = height;
  }

  const unlistenMouseMove = await window.listen<[x: number, y: number]>(SeelenEvent.GlobalMouseMove, (event) => {
    const [mouseX, mouseY] = event.payload;
    const { x: windowX, y: windowY, width: windowWidth, height: windowHeight } = getPhysicalRect
      ? getPhysicalRect()
      : trackedRect;

    const isHoverWindow = mouseX >= windowX &&
      mouseX <= windowX + windowWidth &&
      mouseY >= windowY &&
      mouseY <= windowY + windowHeight;

    if (!isHoverWindow) {
      return;
    }

    const adjustedX = (mouseX - windowX) / globalThis.devicePixelRatio;
    const adjustedY = (mouseY - windowY) / globalThis.devicePixelRatio;

    const elementAtPoint = document.elementFromPoint(adjustedX, adjustedY);
    if (!elementAtPoint) {
      return;
    }

    const shouldAllow = shouldAllowMouseEvent(elementAtPoint);
    if (shouldAllow == data.isIgnoringCursorEvents) {
      data.isIgnoringCursorEvents = !shouldAllow;
      window.setIgnoreCursorEvents(!shouldAllow);
    }
  });

  const onTouchStart = (e: TouchEvent) => {
    const shouldAllow = shouldAllowMouseEvent(e.target as Element);
    if (shouldAllow == data.isIgnoringCursorEvents) {
      data.isIgnoringCursorEvents = !shouldAllow;
      window.setIgnoreCursorEvents(!shouldAllow);
    }
  };
  globalThis.addEventListener("touchstart", onTouchStart);

  return () => {
    unlistenMoved?.();
    unlistenResized?.();
    unlistenMouseMove();
    globalThis.removeEventListener("touchstart", onTouchStart);
    window.setIgnoreCursorEvents(false);
  };
}
