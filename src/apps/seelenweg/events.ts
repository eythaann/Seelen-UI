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
  root_container.style.width =
    mode === SeelenWegMode.FULL_WIDTH ? `calc(100% - ${margin * 2}px)` : 'min-content';

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

export const ExtraCallbacksOnLeave = {
  callbacks: [] as (() => void)[],
  add(cb: () => void) {
    this.callbacks.push(cb);
  },
  execute() {
    this.callbacks.forEach((fn) => fn());
  },
};

export function registerDocumentEvents() {
  const timeoutId: TimeoutIdRef = { current: null };
  const webview = getCurrent();

  document.addEventListener('contextmenu', (event) => event.preventDefault());

  const onMouseLeave = () => {
    webview.setIgnoreCursorEvents(true);
    updateHitbox(); // ensure min size hitbox on unzoom elements
    ExtraCallbacksOnLeave.execute();
  };

  const onMouseEnter = () => {
    if (timeoutId.current) {
      clearTimeout(timeoutId.current);
    }
    webview.setIgnoreCursorEvents(false);
  };

  root_container.addEventListener('mouseleave', debounce(onMouseLeave, 200, timeoutId));

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
