import { debounce, TimeoutIdRef } from '../utils/Timing';
import { getCurrent } from '@tauri-apps/api/webviewWindow';

import { CallbacksManager } from './modules/shared/utils/app';

const root_container = document.getElementById('root')!;

export const ExtraCallbacksOnLeave = new CallbacksManager();
export const ExtraCallbacksOnActivate = new CallbacksManager();

export function registerDocumentEvents() {
  const timeoutId: TimeoutIdRef = { current: null };
  const webview = getCurrent();

  const onMouseLeave = debounce(() => {
    webview.setIgnoreCursorEvents(true);
    ExtraCallbacksOnLeave.execute();
  }, 200, timeoutId);

  const onMouseEnter = () => {
    if (timeoutId.current) {
      clearTimeout(timeoutId.current);
    }
    webview.setIgnoreCursorEvents(false);
    ExtraCallbacksOnActivate.execute();
  };

  root_container.addEventListener('mouseleave', onMouseLeave);
  // if for some reazon mouseleave is not emitted
  // set ignore cursor events when user click on screen
  document.body.addEventListener('click', (event) => {
    if (event.target === document.body) {
      onMouseLeave();
    }
  });

  root_container.addEventListener('mouseenter', onMouseEnter);
  webview.listen('mouseenter', onMouseEnter); // listener for hitbox

  webview.listen<{ x: number; y: number }>('click', (event) => {
    let element = document.elementFromPoint(event.payload.x, event.payload.y);
    if (element && 'click' in element && typeof element.click === 'function') {
      element.click();
    }
  });
}
