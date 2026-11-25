import { invoke, SeelenCommand, SeelenEvent, Settings, subscribe, Widget } from "@seelen-ui/lib";
import type { UserAppWindow } from "@seelen-ui/lib/types";
import { type LazyRune, lazyRune } from "libs/ui/svelte/utils";
import { locale } from "./i18n/index.ts";

const settings = lazyRune(() => Settings.getAsync());
Settings.onChange((s) => (settings.value = s));

// Same rule as the wallpapers: any performance mode other than "Disabled" pauses live rendering
// (static snapshots are shown instead).
const performanceMode = lazyRune(() => invoke(SeelenCommand.StateGetPerformanceMode));
await subscribe(SeelenEvent.StatePerformanceModeChanged, performanceMode.setByPayload);

const interactables = lazyRune<UserAppWindow[]>(() => invoke(SeelenCommand.GetUserAppWindows));
await subscribe(SeelenEvent.UserAppWindowsChanged, interactables.setByPayload);

// The base64 snapshot cache is large: it is only fetched/followed once DWM could not provide
// a live surface for some window.
const previews = lazyRune<Record<string, { data: string }>>(() => invoke(SeelenCommand.GetUserAppWindowsPreviews));
previews.value = {};
let fallbackEnabled = false;
let snapshotRevision = 0;
await subscribe(SeelenEvent.UserAppWindowsPreviewsChanged, (event) => {
  if (fallbackEnabled) {
    snapshotRevision++;
    previews.setByPayload(event);
  }
});

let hwnds = $state<string[]>([]);
// Bumped on every trigger: the popup may be re-triggered for another dock item while visible.
let triggerCount = $state(0);

Widget.self.onTrigger(({ customArgs }) => {
  hwnds = (customArgs?.hwnds as string[]) ?? [];
  triggerCount++;
});

await Promise.all([interactables.init(), settings.init(), performanceMode.init()]);

$effect.root(() => {
  $effect(() => {
    locale.set(settings.value.language);
  });
});

const currentInteractables = $derived(interactables.value?.filter((w) => hwnds.includes(w.hwnd)) ?? []);
const liveEnabled = $derived(performanceMode.value === "Disabled");

class PreviewState {
  enableSnapshotFallback(): void {
    if (fallbackEnabled) return;
    fallbackEnabled = true;
    const revision = snapshotRevision;
    invoke(SeelenCommand.GetUserAppWindowsPreviews).then((value) => {
      if (revision === snapshotRevision) previews.value = value;
    }).catch(console.error);
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

  get previews(): LazyRune<Record<string, { data: string }>> {
    return previews;
  }
}

export const previewState = new PreviewState();
