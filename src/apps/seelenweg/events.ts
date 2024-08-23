import { toPhysicalPixels } from '../shared';
import { AppBarHideMode, SeelenWegSide } from '../shared/schemas/Seelenweg';
import { debounce, TimeoutIdRef } from '../shared/Timing';
import { emitTo } from '@tauri-apps/api/event';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';

import { store } from './modules/shared/store/infra';

import { CallbacksManager } from './modules/shared/utils/app';

const root_container = document.getElementById('root')!;

export const ExtraCallbacksOnLeave = new CallbacksManager();
export const ExtraCallbacksOnActivate = new CallbacksManager();

export const updateHitbox = debounce(async () => {
  const {
    isOverlaped,
    settings: { position, hideMode },
  } = store.getState();
  const view = getCurrentWebviewWindow();
  const hitboxTarget = view.label.replace('/', '-hitbox/');

  const viewPosition = await view.innerPosition();
  const viewSize = await view.innerSize();

  const isAutoHideOn =
    (hideMode !== AppBarHideMode.Never && isOverlaped) || hideMode === AppBarHideMode.Always;
  const isHorizontal = position === SeelenWegSide.TOP || position === SeelenWegSide.BOTTOM;

  const hiddenOffsetTop =
    position === SeelenWegSide.TOP ? viewPosition.y : viewPosition.y + viewSize.height - 1;
  const hiddenOffsetLeft =
    position === SeelenWegSide.LEFT ? viewPosition.x : viewPosition.x + viewSize.width - 1;
  emitTo(hitboxTarget, 'move', {
    x:
      isAutoHideOn && !isHorizontal
        ? hiddenOffsetLeft
        : viewPosition.x + toPhysicalPixels(root_container.offsetLeft),
    y:
      isAutoHideOn && isHorizontal
        ? hiddenOffsetTop
        : viewPosition.y + toPhysicalPixels(root_container.offsetTop),
  });
  emitTo(hitboxTarget, 'resize', {
    width: isAutoHideOn && !isHorizontal ? 1 : toPhysicalPixels(root_container.offsetWidth),
    height: isAutoHideOn && isHorizontal ? 1 : toPhysicalPixels(root_container.offsetHeight),
  });
}, 300);

export function registerDocumentEvents() {
  const timeoutId: TimeoutIdRef = { current: null };
  const webview = getCurrentWebviewWindow();

  const onMouseLeave = debounce(
    () => {
      webview.setIgnoreCursorEvents(true);
      ExtraCallbacksOnLeave.execute();
      updateHitbox();
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
