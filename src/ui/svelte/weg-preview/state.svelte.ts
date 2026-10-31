import { invoke, SeelenCommand, SeelenEvent, Settings, subscribe, Widget } from "@seelen-ui/lib";
import {
  type MediaPlayer,
  SeelenWegSide,
  type UserAppWindow,
  type WegMediaShortcutProfile,
} from "@seelen-ui/lib/types";
import { type LazyRune, lazyRune } from "libs/ui/svelte/utils";
import type { PreviewRect } from "./hover.ts";

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

const sessions = lazyRune(async () => {
  try {
    return await invoke(SeelenCommand.GetMediaSessions);
  } catch (error) {
    console.error("Windows media sessions are unavailable", error);
    return [] as MediaPlayer[];
  }
});
await subscribe(SeelenEvent.MediaSessions, sessions.setByPayload);
const settings = lazyRune(() => Settings.getAsync());
await Settings.onChange((value) => settings.value = value);

await Promise.all([interactables.init(), sessions.init(), settings.init()]);

let hwnds = $state<number[]>([]);
let position = $state<SeelenWegSide>(SeelenWegSide.Bottom);
let hover = $state(false);
let sourceRect = $state<PreviewRect | null>(null);

Widget.self.onTrigger(({ customArgs }) => {
  hwnds = (customArgs?.hwnds as number[]) ?? [];
  position = (customArgs?.position as SeelenWegSide) ?? SeelenWegSide.Bottom;
  hover = customArgs?.hover === true;
  sourceRect = customArgs?.sourceRect as typeof sourceRect ?? null;
});

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
  get hover(): boolean {
    return hover;
  }

  get sourceRect(): PreviewRect | null {
    return sourceRect;
  }
  get sessions(): MediaPlayer[] {
    return sessions.value;
  }

  get shortcuts(): Record<string, WegMediaShortcutProfile> {
    return settings.value.byWidget["@seelen/weg"].mediaShortcuts;
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
