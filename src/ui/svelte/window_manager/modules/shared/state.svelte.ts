import { invoke, SeelenCommand, SeelenEvent, Settings, subscribe } from "@seelen-ui/lib";
import type { FocusedApp, WindowManagerSettings } from "@seelen-ui/lib/types";

import { NodeUtils } from "./utils.ts";
import { lazyRune } from "libs/ui/svelte/utils/LazyRune.svelte.ts";

let layouts = lazyRune(() => invoke(SeelenCommand.WmGetRenderTree));
await subscribe(SeelenEvent.WMTreeChanged, layouts.setByPayload);
await layouts.init();

let interactables = lazyRune(() => invoke(SeelenCommand.GetUserAppWindows));
await subscribe(SeelenEvent.UserAppWindowsChanged, interactables.setByPayload);
await interactables.init();

let forceRepositioning = $state(0);
await subscribe(SeelenEvent.WMForceRetiling, () => {
  forceRepositioning++;
});

let focusedApp = $state<FocusedApp>(await invoke(SeelenCommand.GetFocusedApp));
await subscribe(SeelenEvent.GlobalFocusChanged, (e) => {
  focusedApp = e.payload;
});

let overlayVisible = $derived.by(() => {
  if (focusedApp.isSeelenOverlay) {
    return true;
  }

  for (const layout of Object.values(layouts.value)) {
    if (
      NodeUtils.contains(layout!, focusedApp.hwnd) &&
      !focusedApp.isMaximized &&
      !focusedApp.isFullscreened
    ) {
      return true;
    }
  }

  return false;
});

let settings = $state<WindowManagerSettings>((await Settings.getAsync()).windowManager);
Settings.onChange((s) => (settings = s.windowManager));

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
  get overlayVisible() {
    return overlayVisible;
  }
  get settings() {
    return settings;
  }
}

export const state = new _State();
