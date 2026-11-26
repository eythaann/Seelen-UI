import { invoke, SeelenCommand, SeelenEvent, Settings, subscribe, Widget } from "@seelen-ui/lib";
import type { UserAppWindow } from "@seelen-ui/lib/types";
import { lazyRune } from "libs/ui/svelte/utils";
import { locale } from "./i18n/index.ts";
import { PreviewSnapshots } from "libs/ui/svelte/utils/PreviewSnapshots.svelte.ts";

const settings = lazyRune(() => Settings.getAsync());
Settings.onChange((s) => (settings.value = s));

// Same rule as the wallpapers: any performance mode other than "Disabled" pauses live rendering
// (static snapshots are shown instead).
const performanceMode = lazyRune(() => invoke(SeelenCommand.StateGetPerformanceMode));
await subscribe(SeelenEvent.StatePerformanceModeChanged, performanceMode.setByPayload);

const interactables = lazyRune<UserAppWindow[]>(() => invoke(SeelenCommand.GetUserAppWindows));
await subscribe(SeelenEvent.UserAppWindowsChanged, interactables.setByPayload);

// Snapshot data is fetched only when live composition or performance mode requires it.
const previews = new PreviewSnapshots();
let fallbackEnabled = $state(false);

let hwnds = $state<string[]>([]);
// Bumped on every trigger: the popup may be re-triggered for another dock item while visible.
let triggerCount = $state(0);

Widget.self.onTrigger(({ customArgs }) => {
  hwnds = (customArgs?.hwnds as string[]) ?? [];
  previews.stop();
  fallbackEnabled = false;
  triggerCount++;
});
Widget.self.onVisibilityChange((visible) => {
  if (visible && fallbackEnabled) previews.start(new Set(hwnds));
  if (!visible) previews.stop();
});

await Promise.all([interactables.init(), settings.init(), performanceMode.init()]);

$effect.root(() => {
  $effect(() => {
    locale.set(settings.value.language);
  });
});

const currentInteractables = $derived.by(() => {
  const selected = new Set(hwnds);
  return interactables.value?.filter((window) => selected.has(window.id)) ?? [];
});
const liveEnabled = $derived(performanceMode.value === "Disabled");
const visiblePreviews = $derived.by(() => {
  if (!fallbackEnabled) return {};
  return previews.value;
});

class PreviewState {
  enableSnapshotFallback(): void {
    if (fallbackEnabled) return;
    fallbackEnabled = true;
    if (Widget.self.visible) previews.start(new Set(hwnds));
  }

  get liveEnabled(): boolean {
    return liveEnabled;
  }

  get triggerCount(): number {
    return triggerCount;
  }

  get currentInteractables(): UserAppWindow[] {
    return currentInteractables;
  }

  get previews(): typeof previews.value {
    return visiblePreviews;
  }
}

export const previewState = new PreviewState();
