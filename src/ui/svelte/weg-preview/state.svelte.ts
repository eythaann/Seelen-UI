import { invoke, SeelenCommand, SeelenEvent, Settings, subscribe, Widget } from "@seelen-ui/lib";
import type { Rect as PreviewRect, UserAppWindow } from "@seelen-ui/lib/types";
import { type LazyRune, lazyRune } from "libs/ui/svelte/utils";
import { locale } from "./i18n/index.ts";

const settings = lazyRune(() => Settings.getAsync());
Settings.onChange((s) => (settings.value = s));

const interactables = lazyRune<UserAppWindow[]>(() => invoke(SeelenCommand.GetUserAppWindows));
await subscribe(SeelenEvent.UserAppWindowsChanged, interactables.setByPayload);

// The base64 snapshot cache is large: it is only fetched/followed once DWM could not provide
// a live surface for some window.
const previews = lazyRune<Record<number, { data: string }>>(() => invoke(SeelenCommand.GetUserAppWindowsPreviews));
previews.value = {};
let fallbackEnabled = false;
let snapshotRevision = 0;
await subscribe(SeelenEvent.UserAppWindowsPreviewsChanged, (event) => {
  if (fallbackEnabled) {
    snapshotRevision++;
    previews.setByPayload(event);
  }
});

let hwnds = $state<number[]>([]);
let sourceRect = $state<PreviewRect | null>(null);

Widget.self.onTrigger(({ customArgs }) => {
  hwnds = (customArgs?.hwnds as number[]) ?? [];
  sourceRect = (customArgs?.sourceRect as PreviewRect | undefined) ?? null;
});

await Promise.all([interactables.init(), settings.init()]);

$effect.root(() => {
  $effect(() => {
    locale.set(settings.value.language);
  });
});

const currentInteractables = $derived(interactables.value?.filter((w) => hwnds.includes(w.hwnd)) ?? []);

class PreviewState {
  enableSnapshotFallback(): void {
    if (fallbackEnabled) return;
    fallbackEnabled = true;
    const revision = snapshotRevision;
    invoke(SeelenCommand.GetUserAppWindowsPreviews).then((value) => {
      if (revision === snapshotRevision) previews.value = value;
    }).catch(console.error);
  }

  get sourceRect(): PreviewRect | null {
    return sourceRect;
  }

  get currentInteractables(): UserAppWindow[] {
    return currentInteractables;
  }

  get previews(): LazyRune<Record<number, { data: string }>> {
    return previews;
  }
}

export const previewState = new PreviewState();
