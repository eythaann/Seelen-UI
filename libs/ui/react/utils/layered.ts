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

export interface LayeredHitboxOptions {
  getPhysicalRect: () => LayeredHitboxRect;
  shouldAllowMouseEvent?: (element: Element) => boolean;
}

export async function declareDocumentAsLayeredHitbox(
  options: LayeredHitboxOptions,
): Promise<() => void> {
  const { getPhysicalRect, shouldAllowMouseEvent = (element) => element != document.body } = options;
  const window = TauriWindow.getCurrentWindow();

  await window.setIgnoreCursorEvents(true);
  const data = new LayeredHitbox();

  const unlistenMouseMove = await window.listen<[x: number, y: number]>(SeelenEvent.GlobalMouseMove, (event) => {
    const [mouseX, mouseY] = event.payload;
    const { x: windowX, y: windowY, width: windowWidth, height: windowHeight } = getPhysicalRect();

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
    unlistenMouseMove();
    globalThis.removeEventListener("touchstart", onTouchStart);
    window.setIgnoreCursorEvents(false);
  };
}
