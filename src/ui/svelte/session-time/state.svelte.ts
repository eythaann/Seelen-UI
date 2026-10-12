import { invoke, SeelenCommand, SeelenEvent, subscribe } from "@seelen-ui/lib";
import { SessionTimeSound, type Settings } from "@seelen-ui/lib/types";

type SessionTimeSnapshot = {
  activeSeconds: number;
  longestActiveSeconds: number;
  isIdle: boolean;
};

type SessionTimeCueSettings = {
  enabled: boolean;
  intervalMinutes: number;
  sound: SessionTimeSound;
  customPath: string | null;
  notificationEnabled: boolean;
  notificationMessage: string;
};

let snapshot = $state<SessionTimeSnapshot>({
  activeSeconds: 0,
  longestActiveSeconds: 0,
  isIdle: false,
});
let receivedUpdate = false;
let cueSettings = $state<SessionTimeCueSettings>({
  enabled: false,
  intervalMinutes: 60,
  sound: SessionTimeSound.SystemAsterisk,
  customPath: null,
  notificationEnabled: false,
  notificationMessage: "",
});
let receivedSettingsUpdate = false;

subscribe(SeelenEvent.ActiveSessionTimeChanged, ({ payload }) => {
  snapshot = payload;
  receivedUpdate = true;
});

invoke(SeelenCommand.GetActiveSessionTime).then((initialSnapshot) => {
  if (!receivedUpdate) {
    snapshot = initialSnapshot;
  }
});

function applySettings(settings: Settings) {
  cueSettings = {
    enabled: settings.sessionTimeAudioCueEnabled,
    intervalMinutes: settings.sessionTimeAudioCueIntervalMinutes,
    sound: settings.sessionTimeAudioCueSound,
    customPath: settings.sessionTimeAudioCueCustomPath,
    notificationEnabled: settings.sessionTimeNotificationEnabled,
    notificationMessage: settings.sessionTimeNotificationMessage,
  };
}

subscribe(SeelenEvent.StateSettingsChanged, ({ payload }) => {
  applySettings(payload);
  receivedSettingsUpdate = true;
});

invoke(SeelenCommand.StateGetSettings, { path: null }).then((settings) => {
  if (!receivedSettingsUpdate) {
    applySettings(settings);
  }
});

class SessionTimePopupState {
  get snapshot(): SessionTimeSnapshot {
    return snapshot;
  }

  get cueSettings(): SessionTimeCueSettings {
    return cueSettings;
  }

  set cueSettings(settings: SessionTimeCueSettings) {
    cueSettings = settings;
  }
}

export const sessionTimePopupState = new SessionTimePopupState();
