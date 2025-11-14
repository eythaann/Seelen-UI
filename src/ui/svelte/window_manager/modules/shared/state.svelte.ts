import { invoke, SeelenCommand, SeelenEvent, Settings, subscribe, UIColors, WegItems, Widget } from "@seelen-ui/lib";
import type { FocusedApp, WindowManagerSettings, WmNode } from "@seelen-ui/lib/types";

import { NodeUtils } from "./utils.ts";

let layout = $state<WmNode | null>(null);
await Widget.getCurrent().webview.listen<WmNode>(SeelenEvent.WMSetLayout, (e) => {
  layout = e.payload;
});

const getOpenApps = (items: WegItems) => {
  return items.inner.left
    .concat(items.inner.center)
    .concat(items.inner.right)
    .flatMap((item) => {
      if ("windows" in item) {
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

let openApps = $state(getOpenApps(await WegItems.getNonFiltered()));
WegItems.onChange(async () => {
  openApps = getOpenApps(await WegItems.getNonFiltered());
});

let forceRepositioning = $state(0);
await subscribe(SeelenEvent.WMForceRetiling, () => {
  forceRepositioning++;
});

let focusedApp = $state<FocusedApp>(await invoke(SeelenCommand.GetFocusedApp));
await subscribe(SeelenEvent.GlobalFocusChanged, (e) => {
  focusedApp = e.payload;
});

let overlayVisible = $derived.by(() => {
  return focusedApp.isSeelenOverlay ||
    (!!layout &&
      NodeUtils.contains(layout, focusedApp.hwnd) &&
      !focusedApp.isMaximized &&
      !focusedApp.isFullscreened);
});

let settings = $state<WindowManagerSettings>((await Settings.getAsync()).windowManager);
Settings.onChange((s) => (settings = s.windowManager));

// =================================================
//                  CSS variables
// =================================================

(await UIColors.getAsync()).setAsCssVariables();
UIColors.onChange((colors) => colors.setAsCssVariables());

$effect.root(() => {
  $effect(() => {
    const styles = document.documentElement.style;

    styles.setProperty("--config-padding", `${settings.workspacePadding}px`);
    styles.setProperty("--config-containers-gap", `${settings.workspaceGap}px`);

    styles.setProperty("--config-margin-top", `${settings.workspaceMargin.top}px`);
    styles.setProperty("--config-margin-left", `${settings.workspaceMargin.left}px`);
    styles.setProperty("--config-margin-right", `${settings.workspaceMargin.right}px`);
    styles.setProperty("--config-margin-bottom", `${settings.workspaceMargin.bottom}px`);

    styles.setProperty("--config-border-offset", `${settings.border.offset}px`);
    styles.setProperty("--config-border-width", `${settings.border.width}px`);
  });
});

// =================================================
//               Exported State Getters
// =================================================

export type State = _State;
class _State {
  get layout() {
    return layout;
  }
  get openApps() {
    return openApps;
  }
  get forceRepositioning() {
    return forceRepositioning;
  }
  get focusedApp() {
    return focusedApp;
  }
  get overlayVisible() {
    return overlayVisible;
  }
  get settings() {
    return settings;
  }
}

export const state = new _State();
