import type { MediaDeviceSession } from "@seelen-ui/lib/types";

/** One mixer row: all the audio sessions of a same app. */
export interface MixerApp {
  appId: string;
  /** Instance id of any session of the app; volume and mute commands apply to the whole app. */
  sessionId: string;
  name: string;
  iconPath: string | null;
  isSystem: boolean;
  /** The loudest session, so the row never looks quieter than what is heard. */
  volume: number;
  /** Only when every session is muted. */
  muted: boolean;
}

/** Groups the sessions by app (e.g. Discord plays from several processes), sorted by name. */
export function groupSessionsByApp(sessions: MediaDeviceSession[]): MixerApp[] {
  const apps = new Map<string, MixerApp>();
  for (const session of sessions) {
    const app = apps.get(session.appId);
    if (!app) {
      apps.set(session.appId, {
        appId: session.appId,
        sessionId: session.instanceId,
        name: session.name,
        iconPath: session.iconPath,
        isSystem: session.isSystem,
        volume: session.volume,
        muted: session.muted,
      });
      continue;
    }
    app.volume = Math.max(app.volume, session.volume);
    app.muted = app.muted && session.muted;
  }
  return [...apps.values()].sort((a, b) => a.name.localeCompare(b.name));
}
