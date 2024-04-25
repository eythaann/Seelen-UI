import { debounce, TimeoutIdRef } from '../utils/Timing';
import { emitTo, listen } from '@tauri-apps/api/event';
import { getCurrent } from '@tauri-apps/api/webviewWindow';

import { CallbacksManager } from './modules/shared/utils/app';

const root_container = document.getElementById('root')!;

export const ExtraCallbacksOnLeave = new CallbacksManager();
export const ExtraCallbacksOnActivate = new CallbacksManager();

export const updateHitbox = debounce(() => {
  emitTo('fancy-toolbar-hitbox', 'resize', {
    height: 20,
  });
}, 300);

export function registerDocumentEvents() {
  const timeoutId: TimeoutIdRef = { current: null };
  const webview = getCurrent();

  document.addEventListener('contextmenu', (event) => event.preventDefault());

  const onMouseLeave = debounce(() => {
    webview.setIgnoreCursorEvents(true);
    ExtraCallbacksOnLeave.execute();
    updateHitbox();
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
  listen('mouseenter', onMouseEnter); // listener for hitbox
}
