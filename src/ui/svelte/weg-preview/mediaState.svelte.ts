import { invoke, SeelenCommand, SeelenEvent, Settings, subscribe } from "@seelen-ui/lib";
import type { MediaPlayer, WegMediaShortcutProfile } from "@seelen-ui/lib/types";
import { lazyRune } from "libs/ui/svelte/utils";

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

await Promise.all([sessions.init(), settings.init()]);

class MediaState {
  get sessions(): MediaPlayer[] {
    return sessions.value;
  }

  get shortcuts(): Record<string, WegMediaShortcutProfile> {
    return settings.value.byWidget["@seelen/weg"].mediaShortcuts;
  }
}

export const mediaState = new MediaState();
