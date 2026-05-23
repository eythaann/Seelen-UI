import { SeelenEvent } from "@seelen-ui/lib";
import { window as TauriWindow } from "@seelen-ui/lib/tauri";

class LayeredHitbox {
  private _isIgnoringCursorEvents: boolean = true;
  public firstClick: boolean = true;
  public isLayeredEnabled: boolean = true;

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

export async function declareDocumentAsLayeredHitbox(
  shouldAllowMouseEvent: (element: Element) => boolean = (element) => element != document.body,
): Promise<() => void> {
  const window = TauriWindow.getCurrentWindow();

  const webviewRect = { x: 0, y: 0, width: 0, height: 0 };

  await window.setIgnoreCursorEvents(true);
  const data = new LayeredHitbox();

  const unlistenMoved = await window.onMoved((e) => {
    webviewRect.x = e.payload.x;
    webviewRect.y = e.payload.y;
  });

  const unlistenResized = await window.onResized((e) => {
    webviewRect.width = e.payload.width;
    webviewRect.height = e.payload.height;
  });

  const { x, y } = await window.outerPosition();
  webviewRect.x = x;
  webviewRect.y = y;
  const { width, height } = await window.outerSize();
  webviewRect.width = width;
  webviewRect.height = height;

  const unlistenLayered = await window.listen<boolean>(SeelenEvent.HandleLayeredHitboxes, (event) => {
    data.isLayeredEnabled = event.payload;
  });

  const unlistenMouseMove = await window.listen<[x: number, y: number]>(SeelenEvent.GlobalMouseMove, (event) => {
    if (!data.isLayeredEnabled) {
      return;
    }

    const [mouseX, mouseY] = event.payload;
    const { x: windowX, y: windowY, width: windowWidth, height: windowHeight } = webviewRect;

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
    unlistenMoved();
    unlistenResized();
    unlistenLayered();
    unlistenMouseMove();
    globalThis.removeEventListener("touchstart", onTouchStart);
    window.setIgnoreCursorEvents(false);
  };
}
