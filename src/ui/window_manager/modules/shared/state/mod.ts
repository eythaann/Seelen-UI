import { computed, signal } from '@preact/signals';
import { invoke, SeelenCommand, SeelenEvent, subscribe, WegItems, Widget } from '@seelen-ui/lib';
import { WmNode } from '@seelen-ui/lib/types';

import { NodeUtils } from '../utils';

export const $layout = signal<WmNode | null>(null);
await Widget.getCurrent().webview.listen<WmNode>(SeelenEvent.WMSetLayout, (e) => {
  $layout.value = e.payload;
});

const getOpenApps = (items: WegItems) => {
  return items.inner.left
    .concat(items.inner.center)
    .concat(items.inner.right)
    .flatMap((item) => {
      if ('windows' in item) {
        return [
          {
            path: item.path,
            umid: item.umid,
            windows: item.windows,
          },
        ];
      }
      return [];
    });
};

export const $open_apps = signal(getOpenApps(await WegItems.getNonFiltered()));
WegItems.onChange(async () => {
  $open_apps.value = getOpenApps(await WegItems.getNonFiltered());
});

export const $force_repositioning = signal(0);
await subscribe(SeelenEvent.WMForceRetiling, () => {
  $force_repositioning.value++;
});

export const $focused_app = signal(await invoke(SeelenCommand.GetFocusedApp));
await subscribe(SeelenEvent.GlobalFocusChanged, (e) => {
  $focused_app.value = e.payload;
});

export const $overlay_visible = computed(() => {
  return (
    $focused_app.value.isSeelenOverlay ||
    ($layout.value &&
      NodeUtils.contains($layout.value, $focused_app.value.hwnd) &&
      !$focused_app.value.isMaximized &&
      !$focused_app.value.isFullscreened &&
      !$focused_app.value.isBeingDragged)
  );
});
