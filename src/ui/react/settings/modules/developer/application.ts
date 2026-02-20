import { dialog } from "@seelen-ui/lib/tauri";
import { path } from "@tauri-apps/api";
import { settings } from "../../state/mod";
import { invoke, SeelenCommand } from "@seelen-ui/lib";

export async function simulatePerm(widgetId: string, perm: string): Promise<void> {
  await invoke(SeelenCommand.SimulatePerm, { widgetId, perm });
}

/**
 * Gets the devTools setting
 */
export function getDevTools(): boolean {
  return settings.value.devTools;
}

/**
 * Sets the devTools setting
 */
export function setDevTools(devTools: boolean) {
  settings.value = {
    ...settings.value,
    devTools,
  };
}

/**
 * Loads a custom configuration file
 */
export async function LoadCustomConfigFile() {
  const file = await dialog.open({
    defaultPath: await path.homeDir(),
    multiple: false,
    title: "Select settings file",
    filters: [{ name: "settings", extensions: ["json"] }],
  });

  if (!file) {
    return;
  }

  settings.value = await invoke(SeelenCommand.StateGetSettings, { path: file });
}
