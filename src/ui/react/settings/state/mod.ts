import { invoke, SeelenCommand, SeelenEvent, subscribe } from "@seelen-ui/lib";
import type { IconPackId, ThemeId } from "@seelen-ui/lib/types";
import { batch, computed, effect, signal } from "@preact/signals";
import { Modal } from "antd";
import { monitors } from "./system";
import { cloneDeep } from "lodash";
import i18n from "../i18n";
import { bundledAppConfigs, iconPacks, themes } from "./resources";

export const settings = signal(await invoke(SeelenCommand.StateGetSettings, { path: null }));
const initialSettings = signal(JSON.stringify(settings.value));
subscribe(SeelenEvent.StateSettingsChanged, ({ payload }) => {
  settings.value = payload;
  initialSettings.value = JSON.stringify(payload);
});

export const hasChanges = computed(() => initialSettings.value !== JSON.stringify(settings.value));
export const needRestart = signal(false);

export const appsConfig = computed(() => [...bundledAppConfigs.value, ...settings.value.byApp]);

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

export * from "./resources";
export * from "./system";

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
  i18n.changeLanguage(settings.value.language || "en");
});

/// ===============================================================
/// ==========================ACTIONS==============================
/// ===============================================================

export function restoreToLastSaved() {
  settings.value = JSON.parse(initialSettings.value);
}

export function setActiveIconPacks(payload: IconPackId[]) {
  let active = new Set(payload);

  for (const id of payload) {
    if (!iconPacks.value.some((x) => x.id === id)) {
      active.delete(id);
    }
  }

  settings.value = {
    ...settings.value,
    activeIconPacks: Array.from(active),
  };
}

export function setActiveThemes(payload: ThemeId[]) {
  let active = new Set(payload);

  for (const id of payload) {
    if (!themes.value.some((x) => x.id === id)) {
      active.delete(id);
    }
  }

  settings.value = {
    ...settings.value,
    activeThemes: Array.from(active),
  };
}
