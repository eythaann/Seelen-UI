import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { useEffect, useRef } from 'react';

// TODO remove all this for a the library: Seelen-Core

class CallbacksManager {
  callbacks: Record<string, () => void> = {};

  add(cb: () => void, key: string) {
    this.callbacks[key] = cb;
  }

  remove(key: string) {
    delete this.callbacks[key];
  }

  execute() {
    Object.values(this.callbacks).forEach((cb) => cb());
  }
}

const ExtraCallbacksOnBlur = new CallbacksManager();
const ExtraCallbacksOnFocus = new CallbacksManager();

export function useAppBlur(cb: () => void, deps: any[] = []) {
  const key = useRef(crypto.randomUUID());
  useEffect(() => {
    ExtraCallbacksOnBlur.remove(key.current);
    ExtraCallbacksOnBlur.add(cb, key.current);
    return () => {
      ExtraCallbacksOnBlur.remove(key.current);
    };
  }, deps);
}

export function useAppActivation(cb: () => void, deps: any[] = []) {
  const key = useRef(crypto.randomUUID());
  useEffect(() => {
    ExtraCallbacksOnFocus.remove(key.current);
    ExtraCallbacksOnFocus.add(cb, key.current);
    return () => {
      ExtraCallbacksOnFocus.remove(key.current);
    };
  }, deps);
}

export async function registerDocumentEvents() {
  let appFocused = true;
  const webview = getCurrentWebviewWindow();

  webview.onFocusChanged((event) => {
    appFocused = event.payload;
    webview.setIgnoreCursorEvents(!appFocused);
    if (!appFocused) {
      webview.hide();
      ExtraCallbacksOnBlur.execute();
    } else {
      ExtraCallbacksOnFocus.execute();
    }
  });

  // this is started as true on rust side but to be secure we set it to false
  let ignoring_cursor_events = false;

  const { x, y } = await webview.outerPosition();
  const { width, height } = await webview.outerSize();
  let webviewRect = { x, y, width, height };

  webview.onMoved((e) => {
    webviewRect.x = e.payload.x;
    webviewRect.y = e.payload.y;
  });

  webview.onResized((e) => {
    webviewRect.width = e.payload.width;
    webviewRect.height = e.payload.height;
  });

  webview.listen<[x: number, y: number]>('global-mouse-move', async (event) => {
    if (!(await webview.isVisible())) {
      return;
    }

    const [mouseX, mouseY] = event.payload;
    let { x: windowX, y: windowY, width: windowWidth, height: windowHeight } = webviewRect;

    // check if the mouse is inside the window
    const isHoverWindow =
      mouseX >= windowX &&
      mouseX <= windowX + windowWidth &&
      mouseY >= windowY &&
      mouseY <= windowY + windowHeight;

    if (!isHoverWindow) {
      return;
    }

    const adjustedX = (mouseX - windowX) / window.devicePixelRatio;
    const adjustedY = (mouseY - windowY) / window.devicePixelRatio;

    let element = document.elementFromPoint(adjustedX, adjustedY);
    if (element != document.body && ignoring_cursor_events) {
      webview.setIgnoreCursorEvents(false);
      ignoring_cursor_events = false;
    } else if (element == document.body && (!ignoring_cursor_events || appFocused)) {
      webview.setIgnoreCursorEvents(true);
      ignoring_cursor_events = true;
    }
  });

  document.addEventListener('focusin', (event) => {
    const element = event.target as HTMLElement;
    if (element) {
      element.scrollIntoView({
        behavior: 'smooth',
        block: 'center',
        inline: 'nearest',
      });
    }
  });
}
