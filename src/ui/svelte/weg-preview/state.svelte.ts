import { invoke, SeelenCommand, SeelenEvent, subscribe, Widget } from "@seelen-ui/lib";
import { SeelenWegSide, type UserAppWindow } from "@seelen-ui/lib/types";
import { type LazyRune, lazyRune } from "libs/ui/svelte/utils";
import type { Rect as PreviewRect } from "@seelen-ui/lib/types";

const interactables = lazyRune<UserAppWindow[]>(() => invoke(SeelenCommand.GetUserAppWindows));
await subscribe(SeelenEvent.UserAppWindowsChanged, interactables.setByPayload);

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
let position = $state<SeelenWegSide>(SeelenWegSide.Bottom);
let sourceRect = $state<PreviewRect | null>(null);

Widget.self.onTrigger(({ customArgs }) => {
  hwnds = (customArgs?.hwnds as number[]) ?? [];
  position = (customArgs?.position as SeelenWegSide) ?? SeelenWegSide.Bottom;
  sourceRect = customArgs?.sourceRect as typeof sourceRect ?? null;
});

await interactables.init();

const currentInteractables = $derived(interactables.value?.filter((w) => hwnds.includes(w.hwnd)) ?? []);

class PreviewState {
  enableSnapshotFallback(): void {
    if (fallbackEnabled) return;
    fallbackEnabled = true;
    const revision = snapshotRevision;
    // Only fetch the large base64 cache if DWM could not provide a live surface.
    void invoke(SeelenCommand.GetUserAppWindowsPreviews).then((value) => {
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

  get position(): SeelenWegSide {
    return position;
  }
}

export const previewState = new PreviewState();
