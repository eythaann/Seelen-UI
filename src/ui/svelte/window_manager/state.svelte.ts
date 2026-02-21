import { invoke, SeelenCommand, SeelenEvent, Settings, subscribe } from "@seelen-ui/lib";
import type { FocusedApp, WindowManagerSettings } from "@seelen-ui/lib/types";

import { lazyRune } from "libs/ui/svelte/utils/LazyRune.svelte.ts";

let layouts = lazyRune(() => invoke(SeelenCommand.WmGetRenderTree));
subscribe(SeelenEvent.WMTreeChanged, layouts.setByPayload);

let interactables = lazyRune(() => invoke(SeelenCommand.GetUserAppWindows));
subscribe(SeelenEvent.UserAppWindowsChanged, interactables.setByPayload);

let forceRepositioning = $state(0);
subscribe(SeelenEvent.WMForceRetiling, () => {
  forceRepositioning++;
});

const [focusedAppInit, settingsInit] = await Promise.all([
  invoke(SeelenCommand.GetFocusedApp),
  Settings.getAsync(),
  layouts.init(),
  interactables.init(),
]);

let focusedApp = $state<FocusedApp>(focusedAppInit);
subscribe(SeelenEvent.GlobalFocusChanged, (e) => {
  focusedApp = e.payload;
});

let settings = $state<WindowManagerSettings>(settingsInit.byWidget["@seelen/window-manager"]);
Settings.onChange((s) => (settings = s.byWidget["@seelen/window-manager"]));

// =================================================
//                  CSS variables
// =================================================

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
  getLayout(monitorId: string) {
    return layouts.value[monitorId] || null;
  }
  get forceRepositioning() {
    return forceRepositioning;
  }
  get interactables() {
    return interactables.value;
  }
  get focusedApp() {
    return focusedApp;
  }
  get settings() {
    return settings;
  }
}

export const state = new _State();
