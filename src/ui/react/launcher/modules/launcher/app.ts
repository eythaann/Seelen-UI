import { fs } from "@seelen-ui/lib/tauri";
import type { LauncherHistory } from "@seelen-ui/lib/types";
import { path } from "@tauri-apps/api";
import yaml from "js-yaml";

export async function SaveHistory(history: LauncherHistory) {
  const yaml_route = await path.join(await path.appDataDir(), "history");
  await fs.writeTextFile(yaml_route, yaml.dump(history));
}
