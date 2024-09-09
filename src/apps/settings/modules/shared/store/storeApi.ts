import { UserSettings } from '../../../../../shared.interfaces';
import { VariableConvention } from '../../../../shared/schemas';
import { path } from '@tauri-apps/api';
import { convertFileSrc, invoke } from '@tauri-apps/api/core';
import yaml from 'js-yaml';

import { resolveDataPath } from '../config/infra';
import { dialog, fs } from '../tauri/infra';

export class UserSettingsLoader {
  private _withUserApps: boolean = false;
  private _withLayouts: boolean = false;
  private _withPlaceholders: boolean = false;
  private _withThemes: boolean = false;
  private _withWallpaper: boolean = false;

  withUserApps() {
    this._withUserApps = true;
    return this;
  }

  withLayouts() {
    this._withLayouts = true;
    return this;
  }

  withPlaceholders() {
    this._withPlaceholders = true;
    return this;
  }

  withWallpaper() {
    this._withWallpaper = true;
    return this;
  }

  withThemes() {
    this._withThemes = true;
    return this;
  }

  onlySettings() {
    this._withUserApps = false;
    this._withLayouts = false;
    this._withPlaceholders = false;
    this._withThemes = false;
    this._withWallpaper = false;
    return this;
  }

  async load(customPath?: string): Promise<UserSettings> {
    const userSettings: UserSettings = {
      jsonSettings: await invoke('state_get_settings', { path: customPath }),
      yamlSettings: [],
      themes: [],
      layouts: [],
      placeholders: [],
      env: await invoke('get_user_envs'),
      wallpaper: null,
    };

    if (this._withUserApps) {
      userSettings.yamlSettings = await invoke('state_get_specific_apps_configurations');
    }

    if (this._withThemes) {
      userSettings.themes = await invoke('state_get_themes');
    }

    if (this._withLayouts) {
      userSettings.layouts = await invoke('state_get_layouts');
    }

    if (this._withPlaceholders) {
      userSettings.placeholders = await invoke('state_get_placeholders');
    }

    if (this._withWallpaper) {
      let wallpaper = await invoke<string>('state_get_wallpaper');
      userSettings.wallpaper = wallpaper ? convertFileSrc(wallpaper) : null;
    }

    return userSettings;
  }
}

export async function saveJsonSettings(settings: UserSettings['jsonSettings']) {
  const json_route = await resolveDataPath('settings.json');
  await fs.writeTextFile(
    json_route,
    JSON.stringify(VariableConvention.fromCamelToSnake(settings), null, 2),
  );
}

export async function saveUserSettings(
  settings: Pick<UserSettings, 'jsonSettings' | 'yamlSettings'>,
) {
  const yaml_route = await resolveDataPath('applications.yml');
  await fs.writeTextFile(
    yaml_route,
    yaml.dump(settings.yamlSettings.filter((app) => !app.isBundled)),
  );
  await saveJsonSettings(settings.jsonSettings);
}

export async function ImportApps() {
  const data: any[] = [];

  const files = await dialog.open({
    defaultPath: await path.resolveResource('static/apps_templates'),
    multiple: true,
    title: 'Select template',
    filters: [{ name: 'apps', extensions: ['yaml', 'yml'] }],
  });

  if (!files) {
    return data;
  }

  for (const file of [files].flat()) {
    const processed = yaml.load(await fs.readTextFile(file.path));
    data.push(...(Array.isArray(processed) ? processed : []));
  }

  return data;
}

export async function ExportApps(apps: any[]) {
  const pathToSave = await dialog.save({
    title: 'Exporting Apps',
    defaultPath: await path.join(await path.homeDir(), 'downloads/apps.yml'),
    filters: [{ name: 'apps', extensions: ['yaml', 'yml'] }],
  });
  if (pathToSave) {
    fs.writeTextFile(pathToSave, yaml.dump(apps));
  }
}
