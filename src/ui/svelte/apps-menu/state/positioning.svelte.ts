import { Widget } from "@seelen-ui/lib";
import type { PhysicalMonitor } from "@seelen-ui/lib/types";
import { globalState } from "./mod.svelte";
import { StartView } from "../constants";
import { getAppsMenuRect, resolveTargetMonitor } from "./geometry";

let monitorToShow = $derived.by(() => {
  return resolveTargetMonitor(globalState.monitors, globalState.desiredMonitorId);
});

async function placeCenteredToMonitor(targetMonitor: PhysicalMonitor): Promise<void> {
  await Widget.getCurrent().setPosition(getAppsMenuRect(targetMonitor, globalState.displayMode));
}

$effect.root(() => {
  $effect(() => {
    globalState.displayMode;
    if (monitorToShow) {
      placeCenteredToMonitor(monitorToShow);
    }
  });
});

export async function prepareForTrigger(monitorId?: string | null): Promise<void> {
  globalState.view = StartView.Favorites;
  globalState.desiredMonitorId = monitorId || null;
  globalState.version++; // trigger reactive updates

  if (monitorToShow) {
    await placeCenteredToMonitor(monitorToShow);
  }
}

export async function onTriggered(monitorId?: string | null): Promise<void> {
  await prepareForTrigger(monitorId);
  await Widget.self.show();
  await Widget.self.focus();
}
