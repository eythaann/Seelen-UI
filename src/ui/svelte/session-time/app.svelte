<script lang="ts">
  import { invoke, SeelenCommand } from "@seelen-ui/lib";
  import { dialog } from "@seelen-ui/lib/tauri";
  import { SessionTimeSound } from "@seelen-ui/lib/types";
  import { t } from "./i18n";
  import { sessionTimePopupState } from "./state.svelte";

  let showSettings = $state(false);
  let isSavingSettings = $state(false);
  let isChoosingFile = $state(false);
  let settingsError = $state("");
  let selectedSound = $state(sessionTimePopupState.cueSettings.sound);
  let availableSystemSounds = $state<SessionTimeSound[]>([SessionTimeSound.SystemDefault]);
  let intervalDraft = $state(formatInterval(sessionTimePopupState.cueSettings.intervalMinutes));

  function formatDuration(seconds: number): string {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const remainingSeconds = seconds % 60;
    return `${hours.toString().padStart(2, "0")}:${minutes.toString().padStart(2, "0")}:${remainingSeconds.toString().padStart(2, "0")}`;
  }

  function formatInterval(totalMinutes: number): string {
    const intervalMinutes = Math.max(1, totalMinutes);
    const hours = Math.floor(intervalMinutes / 60);
    const minutes = intervalMinutes % 60;
    if (hours && minutes) return `${hours}h${minutes}m`;
    if (hours) return `${hours}h`;
    return `${minutes}m`;
  }

  function parseInterval(value: string): number | null {
    const input = value.trim().toLowerCase().replace(/\s+/g, "");
    if (/^\d+$/.test(input)) {
      const minutes = Number(input);
      return Number.isSafeInteger(minutes) && minutes > 0 && minutes <= 0xffffffff
        ? minutes
        : null;
    }

    const match = /^(?:(\d+)h)?(?:(\d+)m)?$/.exec(input);
    if (!match || (!match[1] && !match[2])) return null;
    const hours = Number(match[1] ?? 0);
    const minutes = Number(match[2] ?? 0);
    const totalMinutes = hours * 60 + minutes;
    return Number.isSafeInteger(totalMinutes) && totalMinutes > 0 && totalMinutes <= 0xffffffff
      ? totalMinutes
      : null;
  }

  async function saveCueSettings(
    changes: Partial<typeof sessionTimePopupState.cueSettings>,
  ): Promise<boolean> {
    const next = { ...sessionTimePopupState.cueSettings, ...changes };
    isSavingSettings = true;
    settingsError = "";
    try {
      await invoke(SeelenCommand.SetSessionTimeAudioCueSettings, {
        enabled: next.enabled,
        intervalMinutes: Math.max(1, Math.floor(next.intervalMinutes)),
        sound: next.sound,
        customPath: next.customPath,
        notificationEnabled: next.notificationEnabled,
        notificationMessage: next.notificationMessage,
      });
      sessionTimePopupState.cueSettings = {
        ...next,
        intervalMinutes: Math.max(1, Math.floor(next.intervalMinutes)),
      };
      return true;
    } catch (error) {
      settingsError = String(error);
      return false;
    } finally {
      isSavingSettings = false;
    }
  }

  async function selectCustomMp3(): Promise<boolean> {
    settingsError = "";
    isChoosingFile = true;
    try {
      const selected = await dialog.open({
        title: $t("select_sound_file"),
        multiple: false,
        filters: [{ name: "MP3 audio", extensions: ["mp3"] }],
      });
      if (typeof selected === "string") {
        const saved = await saveCueSettings({
          sound: SessionTimeSound.CustomMp3,
          customPath: selected,
        });
        if (saved) selectedSound = SessionTimeSound.CustomMp3;
        return saved;
      }
      return false;
    } catch (error) {
      settingsError = String(error);
      return false;
    } finally {
      isChoosingFile = false;
    }
  }

  async function changeSound(sound: SessionTimeSound): Promise<void> {
    selectedSound = sound;
    if (sound === SessionTimeSound.CustomMp3) {
      if (sessionTimePopupState.cueSettings.customPath) {
        if (!(await saveCueSettings({ sound }))) {
          selectedSound = sessionTimePopupState.cueSettings.sound;
        }
      } else if (!(await selectCustomMp3())) {
        selectedSound = sessionTimePopupState.cueSettings.sound;
      }
      return;
    }
    if (!(await saveCueSettings({ sound }))) {
      selectedSound = sessionTimePopupState.cueSettings.sound;
    }
  }

  async function saveIntervalDraft(): Promise<void> {
    const intervalMinutes = parseInterval(intervalDraft);
    if (intervalMinutes === null) {
      settingsError = $t("invalid_interval");
      return;
    }
    await saveCueSettings({ intervalMinutes });
  }

  function blurIntervalOnEnter(event: KeyboardEvent): void {
    if (event.key === "Enter") {
      event.preventDefault();
      if (event.currentTarget instanceof HTMLInputElement) {
        event.currentTarget.blur();
      }
    }
  }

  async function testCueSound(): Promise<void> {
    settingsError = "";
    try {
      await invoke(SeelenCommand.TestSessionTimeAudioCue, {
        sound: selectedSound,
        customPath: sessionTimePopupState.cueSettings.customPath,
      });
    } catch (error) {
      settingsError = String(error);
    }
  }

  async function loadAvailableSystemSounds(): Promise<void> {
    try {
      availableSystemSounds = await invoke(SeelenCommand.GetSessionTimeAudioCueSounds);
    } catch (error) {
      settingsError = String(error);
    }
  }

  function soundLabelKey(sound: SessionTimeSound): string {
    switch (sound) {
      case SessionTimeSound.SystemAlarm1:
        return "sound_alarm_1";
      case SessionTimeSound.SystemAlarm2:
        return "sound_alarm_2";
      case SessionTimeSound.SystemAlarm3:
        return "sound_alarm_3";
      case SessionTimeSound.SystemAlarm4:
        return "sound_alarm_4";
      case SessionTimeSound.SystemAlarm5:
        return "sound_alarm_5";
      case SessionTimeSound.SystemAlarm6:
        return "sound_alarm_6";
      case SessionTimeSound.SystemAlarm7:
        return "sound_alarm_7";
      case SessionTimeSound.SystemAlarm8:
        return "sound_alarm_8";
      case SessionTimeSound.SystemAlarm9:
        return "sound_alarm_9";
      case SessionTimeSound.SystemAlarm10:
        return "sound_alarm_10";
      case SessionTimeSound.SystemAsterisk:
        return "sound_asterisk";
      case SessionTimeSound.SystemCall1:
        return "sound_call_1";
      case SessionTimeSound.SystemCall2:
        return "sound_call_2";
      case SessionTimeSound.SystemCall3:
        return "sound_call_3";
      case SessionTimeSound.SystemCall4:
        return "sound_call_4";
      case SessionTimeSound.SystemCall5:
        return "sound_call_5";
      case SessionTimeSound.SystemCall6:
        return "sound_call_6";
      case SessionTimeSound.SystemCall7:
        return "sound_call_7";
      case SessionTimeSound.SystemCall8:
        return "sound_call_8";
      case SessionTimeSound.SystemCall9:
        return "sound_call_9";
      case SessionTimeSound.SystemCall10:
        return "sound_call_10";
      case SessionTimeSound.SystemCriticalBattery:
        return "sound_critical_battery";
      case SessionTimeSound.SystemDeviceConnect:
        return "sound_device_connect";
      case SessionTimeSound.SystemDeviceDisconnect:
        return "sound_device_disconnect";
      case SessionTimeSound.SystemDeviceFail:
        return "sound_device_fail";
      case SessionTimeSound.SystemDefault:
        return "sound_default";
      case SessionTimeSound.SystemExclamation:
        return "sound_exclamation";
      case SessionTimeSound.SystemExit:
        return "sound_exit";
      case SessionTimeSound.SystemFaxBeep:
        return "sound_fax";
      case SessionTimeSound.SystemHand:
        return "sound_hand";
      case SessionTimeSound.SystemIm:
        return "sound_im";
      case SessionTimeSound.SystemLogon:
        return "sound_logon";
      case SessionTimeSound.SystemLowBattery:
        return "sound_low_battery";
      case SessionTimeSound.SystemMail:
        return "sound_mail";
      case SessionTimeSound.SystemMessageNudge:
        return "sound_message_nudge";
      case SessionTimeSound.SystemNotification:
        return "sound_notification";
      case SessionTimeSound.SystemQuestion:
        return "sound_question";
      case SessionTimeSound.SystemReminder:
        return "sound_reminder";
      case SessionTimeSound.SystemStart:
        return "sound_start";
      case SessionTimeSound.SystemUac:
        return "sound_uac";
      case SessionTimeSound.SystemUnlock:
        return "sound_unlock";
      case SessionTimeSound.SystemProximity:
        return "sound_proximity";
      case SessionTimeSound.SystemProximityConnection:
        return "sound_proximity_connection";
      case SessionTimeSound.SystemWelcome:
        return "sound_welcome";
      case SessionTimeSound.CustomMp3:
        return "custom_mp3";
    }
  }

  $effect(() => {
    intervalDraft = formatInterval(sessionTimePopupState.cueSettings.intervalMinutes);
  });

  $effect(() => {
    selectedSound = sessionTimePopupState.cueSettings.sound;
  });

  $effect(() => {
    void loadAvailableSystemSounds();
  });
</script>

<div class="slu-std-popover session-time-popup">
  <div class="session-time-heading">{$t("longest_session")}</div>
  <div class="session-time-score">
    {formatDuration(sessionTimePopupState.snapshot.longestActiveSeconds)}
  </div>
  <div class="session-time-current">
    <span>{$t("active_session")}</span>
    <span>{formatDuration(sessionTimePopupState.snapshot.activeSeconds)}</span>
  </div>
  <button
    type="button"
    class="session-time-settings-toggle"
    data-skin="transparent"
    aria-label={$t("settings")}
    title={$t("settings")}
    aria-expanded={showSettings}
    onclick={() => (showSettings = !showSettings)}
  >
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <path
        d="m5 19 8.6-8.6m-1.4-1.4 3.5-3.5 3.4 3.4-3.5 3.5M4 20l3-1-2-2-1 3Zm10-11 1 1M8 5l3 3m-4-5 5 5"
        fill="none"
        stroke="currentColor"
        stroke-width="1.8"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
    </svg>
  </button>
  {#if showSettings}
    <div class="session-time-settings">
      <label class="session-time-setting">
        <span>{$t("audio_cue")}</span>
        <input
          type="checkbox"
          data-skin="switch"
          checked={sessionTimePopupState.cueSettings.enabled}
          disabled={isSavingSettings || isChoosingFile}
          onchange={(event) =>
            saveCueSettings({ enabled: event.currentTarget.checked })}
        />
      </label>
      <label class="session-time-setting">
        <span>{$t("interval")}</span>
        <input
          type="text"
          data-skin="default"
          placeholder={$t("interval_placeholder")}
          aria-label={$t("interval")}
          data-error={settingsError === $t("invalid_interval")}
          value={intervalDraft}
          disabled={isSavingSettings || isChoosingFile}
          oninput={(event) => (intervalDraft = event.currentTarget.value)}
          onchange={saveIntervalDraft}
          onkeydown={blurIntervalOnEnter}
        />
      </label>
      <label class="session-time-setting">
        <span>{$t("sound")}</span>
        <select
          data-skin="default"
          bind:value={selectedSound}
          disabled={isSavingSettings || isChoosingFile}
          onchange={() => changeSound(selectedSound)}
        >
          {#each availableSystemSounds as sound}
            <option value={sound}>
              {$t(soundLabelKey(sound))}
            </option>
          {/each}
          <option value={SessionTimeSound.CustomMp3}>{$t("custom_mp3")}</option>
        </select>
      </label>
      <div class="session-time-custom-sound">
        <button
          type="button"
          class="session-time-file-button"
          data-skin="default"
          disabled={isSavingSettings || isChoosingFile}
          onclick={() => selectCustomMp3()}
        >{$t("choose_mp3")}</button>
        <span>
          {sessionTimePopupState.cueSettings.customPath?.split(/[\\/]/).at(-1) ??
            $t("no_file_selected")}
        </span>
      </div>
      <label class="session-time-setting">
        <span>{$t("notification")}</span>
        <input
          type="checkbox"
          data-skin="switch"
          checked={sessionTimePopupState.cueSettings.notificationEnabled}
          disabled={isSavingSettings || isChoosingFile}
          onchange={(event) =>
            saveCueSettings({ notificationEnabled: event.currentTarget.checked })}
        />
      </label>
      {#if sessionTimePopupState.cueSettings.notificationEnabled}
        <label class="session-time-setting">
          <span>{$t("notification_message")}</span>
          <input
            type="text"
            data-skin="default"
            maxlength="200"
            placeholder={$t("notification_message_placeholder")}
            aria-label={$t("notification_message")}
            value={sessionTimePopupState.cueSettings.notificationMessage}
            disabled={isSavingSettings || isChoosingFile}
            onchange={(event) =>
              saveCueSettings({ notificationMessage: event.currentTarget.value })}
            onkeydown={blurIntervalOnEnter}
          />
        </label>
      {/if}
      <button
        type="button"
        class="session-time-test-button"
        data-skin="default"
        disabled={isSavingSettings}
        onclick={testCueSound}
      >{$t("test_sound")}</button>
      {#if settingsError}
        <div class="session-time-settings-error" role="alert">{settingsError}</div>
      {/if}
    </div>
  {/if}
</div>
