import { SeelenEvent } from '@seelen-ui/lib';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';

export function disableWebviewShortcutsAndContextMenu(): void {
  globalThis.addEventListener('keydown', function (event): void {
    // Prevent refresh
    if (event.key === 'F5' || (event.ctrlKey && event.key === 'R')) {
      event.preventDefault();
    }

    // Prevent closing the window (Alt+F4 / Cmd+Q on macOS)
    if ((event.altKey && event.key === 'F4') || (event.metaKey && event.key === 'q')) {
      event.preventDefault();
    }

    // Prevent common Ctrl/Cmd shortcuts
    if (event.ctrlKey || event.metaKey) {
      switch (event.key) {
        case 'n': // New window
        case 't': // New tab
        case 'w': // Close tab
        case 'f': // Find
        case 'g': // Find next
        case 'p': // print
        case 's': // Save
        case 'o': // Open file
        case 'j': // downloads
        case 'u': // View source
        case 'tab': // Switch tabs
          event.preventDefault();
          break;
      }
    }
  });

  // prevent browser context menu
  globalThis.addEventListener('contextmenu', (e) => e.preventDefault(), {
    capture: true,
  });

  // Prevent drag-and-drop (files, links, images)
  globalThis.addEventListener('drop', (e) => e.preventDefault());
  globalThis.addEventListener('dragover', (e) => e.preventDefault());
  globalThis.addEventListener('dragstart', (e) => e.preventDefault());
}

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

export async function declareDocumentAsLayeredHitbox(): Promise<void> {
  const webview = getCurrentWebviewWindow();
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

    const isOverBody = document.elementFromPoint(adjustedX, adjustedY) == document.body;
    if (isOverBody && !data.isIgnoringCursorEvents) {
      data.isIgnoringCursorEvents = true;
      webview.setIgnoreCursorEvents(true);
    }

    if (!isOverBody && data.isIgnoringCursorEvents) {
      data.isIgnoringCursorEvents = false;
      webview.setIgnoreCursorEvents(false);
    }
  });

  globalThis.addEventListener('touchstart', (e) => {
    const isOverBody = e.target == document.body;
    if (isOverBody && !data.isIgnoringCursorEvents) {
      data.isIgnoringCursorEvents = true;
      webview.setIgnoreCursorEvents(data.isLayeredEnabled);
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
