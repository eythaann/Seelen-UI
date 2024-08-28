import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';

import { CallbacksManager } from './modules/shared/utils/app';

export const ExtraCallbacksOnBlur = new CallbacksManager();
export const ExtraCallbacksOnFocus = new CallbacksManager();

export function registerDocumentEvents() {
  let appFocused = true;
  const webview = getCurrentWebviewWindow();

  function onAppBlur() {
    appFocused = false;
    webview.setIgnoreCursorEvents(true);
    ExtraCallbacksOnBlur.execute();
  }

  function onAppFocus() {
    appFocused = true;
    webview.setIgnoreCursorEvents(false);
    ExtraCallbacksOnFocus.execute();
  }

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
    } else if (element == document.body && (!ignoring_cursor_events || appFocused)) {
      webview.setIgnoreCursorEvents(true);
      ignoring_cursor_events = true;
    }
  });
}
