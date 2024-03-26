import { debounce, TimeoutIdRef } from '../Timing';
import { PhysicalSize } from '@tauri-apps/api/dpi';
import { emitTo, listen } from '@tauri-apps/api/event';
import { getCurrent } from '@tauri-apps/api/webviewWindow';

import { store } from './modules/shared/store/infra';

import { SeelenWegMode } from '../settings/modules/seelenweg/domain';

const root_container = document.getElementById('root')!;

export const setWindowSize = () => {
  const webview = getCurrent();
  const screenWidth = Math.floor(window.screen.width * window.devicePixelRatio);
  const screenHeight = Math.floor(window.screen.height * window.devicePixelRatio);
  webview.setSize(new PhysicalSize(screenWidth, screenHeight));
};

export const updateHitbox = () => {
  const { margin, mode } = store.getState().settings;

  root_container.style.margin = margin + 'px';
  root_container.style.width = mode === SeelenWegMode.FULL_WIDTH ? `calc(100% - ${margin * 2}px)` : 'min-content';

  const width = root_container.offsetWidth;
  const height = root_container.offsetHeight;

  const windowWidth = Math.floor(width * window.devicePixelRatio);
  const windowHeight = Math.floor(height * window.devicePixelRatio);

  const screenWidth = Math.floor(window.screen.width * window.devicePixelRatio);
  const screenHeight = Math.floor(window.screen.height * window.devicePixelRatio);

  emitTo('seelenweg-hitbox', 'resize', { width: windowWidth, height: windowHeight });
  emitTo('seelenweg-hitbox', 'move', {
    x: Math.floor(screenWidth / 2 - windowWidth / 2),
    y: Math.floor(screenHeight - windowHeight - Math.round(margin * window.devicePixelRatio)),
  });
};

export function registerDocumentEvents() {
  const timeoutId: TimeoutIdRef = { ref: null };

  document.body.addEventListener('click', (event) => {
    if (event.target === document.body) {
      // if for some reazon mouseleave is not emitted
      // set ignore cursor events when user click on screen
      getCurrent().setIgnoreCursorEvents(true);
    }
  });

  root_container.addEventListener('mouseleave', debounce(() => {
    getCurrent().setIgnoreCursorEvents(true);
    updateHitbox(); // ensure min size hitbox on unzoom elements
  }, 200, timeoutId));

  listen('mouseenter', () => {
    if (timeoutId.ref) {
      clearTimeout(timeoutId.ref);
    }
    getCurrent().setIgnoreCursorEvents(false);
  });
}