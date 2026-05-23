import { listen } from "@tauri-apps/api/event";
import { invoke, RuntimeStyleSheet, SeelenCommand, SeelenEvent, Settings, subscribe, Widget } from "@seelen-ui/lib";
import type { FocusedApp, TwmReservation, TwmRuntimeTree, WindowManagerSettings } from "@seelen-ui/lib/types";
import { FancyToolbarSide, HideMode } from "@seelen-ui/lib/types";
import { SeelenWegSide } from "node_modules/@seelen-ui/lib/esm/gen/types/SeelenWegSide";

import { lazyRune } from "libs/ui/svelte/utils/LazyRune.svelte.ts";

let layouts = lazyRune(() => invoke(SeelenCommand.WmGetRenderTree));
subscribe(SeelenEvent.WMTreeChanged, layouts.setByPayload);

let workspaces = lazyRune(() => invoke(SeelenCommand.StateGetVirtualDesktops));
subscribe(SeelenEvent.VirtualDesktopsChanged, workspaces.setByPayload);

let interactables = lazyRune(() => invoke(SeelenCommand.GetUserAppWindows));
subscribe(SeelenEvent.UserAppWindowsChanged, interactables.setByPayload);

let monitors = lazyRune(() => invoke(SeelenCommand.SystemGetMonitors));
subscribe(SeelenEvent.SystemMonitorsChanged, monitors.setByPayload);

let reservation = $state<TwmReservation | null>(null);
subscribe(SeelenEvent.WMSetReservation, (e) => {
  reservation = e.payload;
});

let forceRepositioning = $state(0);
subscribe(SeelenEvent.WMForceRetiling, () => {
  forceRepositioning++;
});

let paused = $state(false);
listen("internal:twm-toggle-pause", () => {
  paused = !paused;
});

const [focusedAppInit, settingsInit] = await Promise.all([
  invoke(SeelenCommand.GetFocusedApp),
  Settings.getAsync(),
  layouts.init(),
  workspaces.init(),
  interactables.init(),
  monitors.init(),
]);

let focusedApp = $state<FocusedApp>(focusedAppInit);
subscribe(SeelenEvent.GlobalFocusChanged, (e) => {
  focusedApp = e.payload;
});

let fullSettings = $state(settingsInit);
let settings = $state<WindowManagerSettings>(settingsInit.byWidget["@seelen/window-manager"]);
Settings.onChange((s) => {
  fullSettings = s;
  settings = s.byWidget["@seelen/window-manager"];
});

// =================================================
//                  CSS variables
// =================================================

$effect.root(() => {
  $effect(() => {
    const sheet = new RuntimeStyleSheet("@config/window-manager");
    sheet.addVariable("--config-padding", `${settings.workspacePadding}px`);
    sheet.addVariable("--config-containers-gap", `${settings.workspaceGap}px`);
    sheet.addVariable("--config-margin-top", `${settings.workspaceMargin.top}px`);
    sheet.addVariable("--config-margin-left", `${settings.workspaceMargin.left}px`);
    sheet.addVariable("--config-margin-right", `${settings.workspaceMargin.right}px`);
    sheet.addVariable("--config-margin-bottom", `${settings.workspaceMargin.bottom}px`);
    sheet.addVariable("--config-border-offset", `${settings.border.offset}px`);
    sheet.addVariable("--config-border-width", `${settings.border.width}px`);
    sheet.applyToDocument();
  });
});

// =================================================
//                   Positioning
// =================================================

const monitorId = Widget.getCurrent().decoded.monitorId;

$effect.root(() => {
  $effect(() => {
    const monitor = monitors.value.find((m) => m.id === monitorId);
    if (!monitor) return;

    const rect = { ...monitor.rect };
    const tbConfig = fullSettings.byWidget["@seelen/fancy-toolbar"];
    const tbMonitorConfig = (fullSettings.monitorsV3[monitor.id] as any)?.byWidget?.["@seelen/fancy-toolbar"] ?? {
      enabled: true,
    };

    if (tbConfig.enabled && tbMonitorConfig.enabled && tbConfig.hideMode === HideMode.Never) {
      const tbSize = Math.round(
        (tbConfig.itemSize + tbConfig.padding * 2 + tbConfig.margin * 2) * monitor.scaleFactor,
      );
      switch (tbConfig.position) {
        case FancyToolbarSide.Top:
          rect.top += tbSize;
          break;
        case FancyToolbarSide.Bottom:
          rect.bottom -= tbSize;
          break;
      }
    }

    const wegConfig = fullSettings.byWidget["@seelen/weg"];
    const wegMonitorConfig = (fullSettings.monitorsV3[monitor.id] as any)?.byWidget?.["@seelen/weg"] ?? {
      enabled: true,
    };

    if (wegConfig.enabled && wegMonitorConfig.enabled && wegConfig.hideMode === HideMode.Never) {
      const wegSize = Math.round(
        (wegConfig.size + wegConfig.padding * 2 + wegConfig.margin * 2) * monitor.scaleFactor,
      );
      switch (wegConfig.position) {
        case SeelenWegSide.Top:
          rect.top += wegSize;
          break;
        case SeelenWegSide.Bottom:
          rect.bottom -= wegSize;
          break;
        case SeelenWegSide.Left:
          rect.left += wegSize;
          break;
        case SeelenWegSide.Right:
          rect.right -= wegSize;
          break;
      }
    }

    Widget.getCurrent().setPosition(rect);
  });
});

// =================================================
//               Exported State Getters
// =================================================

export type State = _State;
class _State {
  getLayout(monitorId: string): TwmRuntimeTree | null {
    const activeWsId = workspaces.value?.monitors?.[monitorId]?.active_workspace;
    if (!activeWsId) return null;
    return layouts.value?.workspaces?.[activeWsId] ?? null;
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
  get reservation() {
    return reservation;
  }
  get paused() {
    return paused;
  }
}

export const state = new _State();
