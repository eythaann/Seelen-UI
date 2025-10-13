import { dialog } from "@seelen-ui/lib/tauri";
import { path } from "@tauri-apps/api";

import { LoadSettingsToStore } from "../shared/store/infra.ts";

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

  LoadSettingsToStore(file);
}
