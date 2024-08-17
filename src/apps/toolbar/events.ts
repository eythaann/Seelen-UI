import { debounce, TimeoutIdRef } from '../shared/Timing';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';

import { CallbacksManager } from './modules/shared/utils/app';

export const ExtraCallbacksOnLeave = new CallbacksManager();
export const ExtraCallbacksOnActivate = new CallbacksManager();

export function registerDocumentEvents(container: HTMLElement) {
  const timeoutId: TimeoutIdRef = { current: null };
  const webview = getCurrentWebviewWindow();

  const onMouseLeave = debounce(
    () => {
      webview.setIgnoreCursorEvents(true);
      ExtraCallbacksOnLeave.execute();
    },
    200,
    timeoutId,
  );

  const onMouseEnter = () => {
    if (timeoutId.current) {
      clearTimeout(timeoutId.current);
    }
    webview.setIgnoreCursorEvents(false);
    ExtraCallbacksOnActivate.execute();
  };

  container.addEventListener('mouseleave', onMouseLeave);
  // if for some reazon mouseleave is not emitted
  // set ignore cursor events when user click on screen
  document.body.addEventListener('click', (event) => {
    if (event.target === document.body) {
      onMouseLeave();
    }
  });

  container.addEventListener('mouseenter', onMouseEnter);
  webview.listen('mouseenter', onMouseEnter); // listener for hitbox

  webview.listen<{ x: number; y: number }>('click', (event) => {
    let element = document.elementFromPoint(event.payload.x, event.payload.y);
    if (element && 'click' in element && typeof element.click === 'function') {
      element.click();
    }
  });
}
