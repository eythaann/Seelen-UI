import { SeelenCommand } from "@seelen-ui/lib";
import { dialog, fs } from "@seelen-ui/lib/tauri";
import { AppConfig, Settings } from "@seelen-ui/lib/types";
import { path } from "@tauri-apps/api";
import { invoke } from "@tauri-apps/api/core";
import yaml from "js-yaml";

import { resolveDataPath } from "../config/infra";

import i18n from "../../../i18n";

export async function saveJsonSettings(settings: Settings) {
  await invoke(SeelenCommand.StateWriteSettings, { settings });
}

export async function saveUserSettings(settings: {
  jsonSettings: Settings;
  yamlSettings: AppConfig[];
}) {
  const yaml_route = await resolveDataPath("applications.yml");
  await fs.writeTextFile(
    yaml_route,
    yaml.dump(settings.yamlSettings.filter((app) => !app.isBundled)),
  );
  await saveJsonSettings(settings.jsonSettings);
}

export async function ImportApps() {
  const data: any[] = [];

  const files = await dialog.open({
    defaultPath: await path.resolveResource("static/apps_templates"),
    multiple: true,
    title: i18n.t("apps_configurations.import_full"),
    filters: [{ name: "YAML", extensions: ["yaml", "yml"] }],
  });

  if (!files) {
    return data;
  }

  for (const file of [files].flat()) {
    const processed = yaml.load(await fs.readTextFile(file));
    data.push(...(Array.isArray(processed) ? processed : []));
  }

  return data;
}

export async function ExportApps(apps: any[]) {
  const pathToSave = await dialog.save({
    title: i18n.t("apps_configurations.export_full"),
    defaultPath: await path.join(await path.homeDir(), "downloads/apps.yml"),
    filters: [{ name: "YAML", extensions: ["yaml", "yml"] }],
  });
  if (pathToSave) {
    fs.writeTextFile(pathToSave, yaml.dump(apps));
  }
}

export async function ExportResource(resource: { id: string }) {
  const parts = resource.id.split("/");
  const resourceName = parts.pop();
  if (!resourceName) {
    console.error("No resource name when exporting");
    return;
  }
  const date = new Date();
  const filename = resourceName + "." + date.toISOString().split("T")[0] +
    ".yml";
  const pathToSave = await dialog.save({
    title: i18n.t("resources.export"),
    defaultPath: await path.join(await path.downloadDir(), filename),
    filters: [{ name: "YAML", extensions: ["yaml", "yml"] }],
  });
  if (pathToSave) {
    fs.writeTextFile(pathToSave, yaml.dump(resource));
  }
}
