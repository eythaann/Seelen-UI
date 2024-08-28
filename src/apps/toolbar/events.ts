import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';

import { CallbacksManager } from './modules/shared/utils/app';

export const ExtraCallbacksOnBlur = new CallbacksManager();
export const ExtraCallbacksOnFocus = new CallbacksManager();

export function registerDocumentEvents() {
  const webview = getCurrentWebviewWindow();

  function onAppBlur() {
    webview.setIgnoreCursorEvents(true);
    ExtraCallbacksOnBlur.execute();
  }

  function onAppFocus() {
    webview.setIgnoreCursorEvents(false);
    ExtraCallbacksOnFocus.execute();
  }

  // TODO handle touches
  /* webview.listen<{ x: number; y: number }>('click', (event) => {
    let element = document.elementFromPoint(event.payload.x, event.payload.y);
    if (element && 'click' in element && typeof element.click === 'function') {
      element.click();
    }
  }); */

  webview.onFocusChanged((event) => {
    if (event.payload) {
      onAppFocus();
    } else {
      onAppBlur();
    }
  });

  // this is started as true on rust side but to be secure we set it to false
  let ignoring_cursor_events = false;
  webview.listen<[x: number, y: number]>('global-mouse-move', async (event) => {
    if (!(await webview.isVisible())) {
      return;
    }

    const [x, y] = event.payload;
    const adjustedX = x / window.devicePixelRatio;
    const adjustedY = y / window.devicePixelRatio;

    let element = document.elementFromPoint(adjustedX, adjustedY);
    if (element != document.body && ignoring_cursor_events) {
      webview.setIgnoreCursorEvents(false);
      ignoring_cursor_events = false;
    } else if (element == document.body && !ignoring_cursor_events) {
      webview.setIgnoreCursorEvents(true);
      ignoring_cursor_events = true;
    }
  });
}
