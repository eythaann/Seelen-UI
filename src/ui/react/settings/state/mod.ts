import { invoke, SeelenCommand, SeelenEvent, subscribe } from "@seelen-ui/lib";
import { batch, computed, effect, signal } from "@preact/signals";
import { Modal } from "antd";
import { monitors } from "./system";
import { cloneDeep } from "lodash";
import i18n from "../i18n";

export const settings = signal(await invoke(SeelenCommand.StateGetSettings, { path: null }));
const initialSettings = signal(JSON.stringify(settings.value));
subscribe(SeelenEvent.StateSettingsChanged, ({ payload }) => {
  settings.value = payload;
  initialSettings.value = JSON.stringify(payload);
});

export const language = computed(() => settings.value.language || "en");

export const hasChanges = computed(() => initialSettings.value !== JSON.stringify(settings.value));
export const needRestart = signal(false);

const bundledAppConfigs = await invoke(SeelenCommand.StateGetSettingsByApp);
export const appsConfig = computed(() => [...bundledAppConfigs, ...settings.value.byApp]);

export async function saveSettings() {
  try {
    initialSettings.value = JSON.stringify(settings.value);
    await invoke(SeelenCommand.StateWriteSettings, {
      settings: settings.value,
    });
  } catch (error) {
    Modal.error({
      title: "Error on Save",
      content: String(error),
      centered: true,
    });
  }
}

const defaultMonitorConfig = await invoke(SeelenCommand.StateGetDefaultMonitorSettings);
effect(() => {
  const sanitized = settings.peek();
  for (const monitor of monitors.value) {
    if (!sanitized.monitorsV3[monitor.id]) {
      sanitized.monitorsV3[monitor.id] = cloneDeep(defaultMonitorConfig);
    }
  }
  batch(() => {
    settings.value = sanitized;
    initialSettings.value = JSON.stringify(sanitized);
  });
});

effect(() => {
  i18n.changeLanguage(language.value);
});

export function restoreToLastSaved() {
  settings.value = JSON.parse(initialSettings.value);
}
