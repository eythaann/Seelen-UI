import { toPhysicalPixels } from '../utils';
import { SeelenWegHideMode, SeelenWegSide } from '../utils/schemas/Seelenweg';
import { debounce, TimeoutIdRef } from '../utils/Timing';
import { PhysicalSize } from '@tauri-apps/api/dpi';
import { emitTo } from '@tauri-apps/api/event';
import { getCurrent } from '@tauri-apps/api/webviewWindow';

import { store } from './modules/shared/store/infra';

import { CallbacksManager } from './modules/shared/utils/app';

const root_container = document.getElementById('root')!;

export const ExtraCallbacksOnLeave = new CallbacksManager();
export const ExtraCallbacksOnActivate = new CallbacksManager();

export const setWindowSize = () => {
  const webview = getCurrent();
  const screenWidth = Math.floor(window.screen.width * window.devicePixelRatio);
  const screenHeight = Math.floor(window.screen.height * window.devicePixelRatio);
  webview.setSize(new PhysicalSize(screenWidth, screenHeight));
};

export const updateHitbox = debounce(() => {
  const { isOverlaped, settings: { position, hideMode } } = store.getState();

  const isAutoHideOn = (hideMode !== SeelenWegHideMode.Never && isOverlaped) || hideMode === SeelenWegHideMode.Always;
  const isHorizontal = position === SeelenWegSide.TOP || position === SeelenWegSide.BOTTOM;

  const hiddenOffsetTop = position === SeelenWegSide.TOP ? 0 : toPhysicalPixels(window.screen.height) - 1;
  const hiddenOffsetLeft = position === SeelenWegSide.LEFT ? 0 : toPhysicalPixels(window.screen.width) - 1;
  emitTo('seelenweg-hitbox', 'move', {
    x: isAutoHideOn && !isHorizontal ? hiddenOffsetLeft : toPhysicalPixels(root_container.offsetLeft),
    y: isAutoHideOn && isHorizontal ? hiddenOffsetTop : toPhysicalPixels(root_container.offsetTop),
  });
  emitTo('seelenweg-hitbox', 'resize', {
    width: isAutoHideOn && !isHorizontal ? 1 : toPhysicalPixels(root_container.offsetWidth),
    height: isAutoHideOn && isHorizontal ? 1 : toPhysicalPixels(root_container.offsetHeight),
  });
}, 300);

export function registerDocumentEvents() {
  const timeoutId: TimeoutIdRef = { current: null };
  const webview = getCurrent();

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
  webview.listen('mouseenter', onMouseEnter); // listener for hitbox
}
