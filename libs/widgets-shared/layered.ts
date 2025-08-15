import { SeelenEvent } from '@seelen-ui/lib';
import { window as TauriWindow } from '@seelen-ui/lib/tauri';

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
): Promise<void> {
  const webview = TauriWindow.getCurrentWindow();
  const { x, y } = await webview.outerPosition();
  const { width, height } = await webview.outerSize();

  const webviewRect = { x, y, width, height };

  await webview.setIgnoreCursorEvents(true);
  const data = new LayeredHitbox();

  webview.onMoved((e) => {
    webviewRect.x = e.payload.x;
    webviewRect.y = e.payload.y;
  });

  webview.onResized((e) => {
    webviewRect.width = e.payload.width;
    webviewRect.height = e.payload.height;
  });

  webview.listen<boolean>(SeelenEvent.HandleLayeredHitboxes, (event) => {
    data.isLayeredEnabled = event.payload;
  });

  webview.listen<[x: number, y: number]>(SeelenEvent.GlobalMouseMove, (event) => {
    if (!data.isLayeredEnabled) {
      return;
    }

    const [mouseX, mouseY] = event.payload;
    const { x: windowX, y: windowY, width: windowWidth, height: windowHeight } = webviewRect;

    // check if the mouse is inside the window
    const isHoverWindow =
      mouseX >= windowX &&
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
      webview.setIgnoreCursorEvents(!shouldAllow);
    }
  });

  globalThis.addEventListener('touchstart', (e) => {
    const shouldAllow = shouldAllowMouseEvent(e.target as Element);
    if (shouldAllow == data.isIgnoringCursorEvents) {
      data.isIgnoringCursorEvents = !shouldAllow;
      webview.setIgnoreCursorEvents(!shouldAllow);
    }
  });

  // the purpose of this is avoid #662 and #138
  /* const fastToggleIgnoreCursor = debounce(() => {
    webview.setIgnoreCursorEvents(true);
    setTimeout(() => {
      webview.setIgnoreCursorEvents(data.isIgnoringCursorEvents);
    }, 100);
  }, 100);

  globalThis.addEventListener(
    'mouseup',
    () => {
      fastToggleIgnoreCursor();
    },
    true,
  ); */
}
