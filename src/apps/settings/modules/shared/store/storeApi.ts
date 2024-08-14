import { UserSettings } from '../../../../../shared.interfaces';
import { parseAsCamel, safeParseAsCamel, VariableConvention } from '../../../../shared/schemas';
import { Layout, LayoutSchema } from '../../../../shared/schemas/Layout';
import { ParsePlaceholder } from '../../../../shared/schemas/Placeholders';
import { SettingsSchema } from '../../../../shared/schemas/Settings';
import { path } from '@tauri-apps/api';
import { invoke } from '@tauri-apps/api/core';
import { DirEntry } from '@tauri-apps/plugin-fs';
import yaml from 'js-yaml';

import { resolveDataPath } from '../config/infra';
import { dialog, fs } from '../tauri/infra';

interface Entry extends DirEntry {
  path: string;
}

async function getEntries(folderName: string) {
  const bundledPath = await path.join(await path.resourceDir(), 'static', folderName);
  const userPath = await path.join(await path.appDataDir(), folderName);

  const entries: Entry[] = [];

  for (const entry of await fs.readDir(bundledPath)) {
    entries.push({
      ...entry,
      path: await path.join(bundledPath, entry.name),
    });
  }

  for (const entry of await fs.readDir(userPath)) {
    entries.push({
      ...entry,
      path: await path.join(userPath, entry.name),
    });
  }

  return entries;
}

async function loadUserLayouts(ref: UserSettings) {
  const defaultLayout = ref.jsonSettings.windowManager.defaultLayout;
  let found = false;

  for (const entry of await getEntries('layouts')) {
    if (entry.isFile && entry.name.endsWith('.json')) {
      let layout: Layout = JSON.parse(await fs.readTextFile(entry.path));

      layout = safeParseAsCamel(LayoutSchema, layout);
      if (!layout) {
        continue;
      }

      const sanitizedLayout: Layout = {
        ...layout,
        info: {
          ...layout.info,
          filename: entry.name,
        },
      };

      if (sanitizedLayout.info.displayName === 'Unknown') {
        sanitizedLayout.info.displayName = entry.name;
      }

      if (defaultLayout === entry.name) {
        found = true;
      }

      ref.layouts.push(sanitizedLayout);
    }
  }

  if (!found) {
    ref.jsonSettings.windowManager.defaultLayout = ref.layouts[0]?.info.filename || null;
  }
}

async function loadUserPlaceholders(ref: UserSettings) {
  const selectedPlaceholder = ref.jsonSettings.fancyToolbar.placeholder;

  const rawPlaceholders: any[] = await invoke('state_get_placeholders');
  for (const rawPlaceholder of rawPlaceholders) {
    let placeholder = ParsePlaceholder(rawPlaceholder);
    if (placeholder) {
      ref.placeholders.push(placeholder);
    }
  }

  let usingPlaceholder = ref.placeholders.find((x) => x.info.filename === selectedPlaceholder);
  if (!usingPlaceholder) {
    ref.jsonSettings.fancyToolbar.placeholder = ref.placeholders[0]?.info.filename || null;
  }
}

export class UserSettingsLoader {
  private _withUserApps: boolean = false;
  private _withLayouts: boolean = false;
  private _withPlaceholders: boolean = false;
  private _withThemes: boolean = true;

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

  withThemes(value: boolean) {
    this._withThemes = value;
    return this;
  }

  async load(customPath?: string): Promise<UserSettings> {
    const userSettings: UserSettings = {
      jsonSettings: parseAsCamel(SettingsSchema, {}),
      yamlSettings: [],
      themes: [],
      layouts: [],
      placeholders: [],
      env: await invoke('get_user_envs'),
    };

    let data =
      customPath && (await fs.exists(customPath))
        ? JSON.parse(await fs.readTextFile(customPath))
        : await invoke('state_get_settings');

    userSettings.jsonSettings = parseAsCamel(SettingsSchema, data);

    if (this._withUserApps) {
      userSettings.yamlSettings = await invoke('state_get_specific_apps_configurations');
    }

    if (this._withThemes) {
      userSettings.themes = await invoke('state_get_themes');
    }

    if (this._withLayouts) {
      await loadUserLayouts(userSettings);
    }

    if (this._withPlaceholders) {
      await loadUserPlaceholders(userSettings);
    }

    return userSettings;
  }
}

export async function saveJsonSettings(settings: UserSettings['jsonSettings']) {
  const json_route = await resolveDataPath('settings.json');
  await fs.writeTextFile(json_route, JSON.stringify(VariableConvention.fromCamelToSnake(settings), null, 2));
}

export async function saveUserSettings(settings: Pick<UserSettings, 'jsonSettings' | 'yamlSettings'>) {
  const yaml_route = await resolveDataPath('applications.yml');

  await saveJsonSettings(settings.jsonSettings);

  await fs.writeTextFile(yaml_route, yaml.dump(settings.yamlSettings));

  await invoke('refresh_state');

  if (settings.jsonSettings.ahkEnabled) {
    await invoke('start_seelen_shortcuts');
  } else {
    await invoke('kill_seelen_shortcuts');
  }
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
    defaultPath: await path.join(await path.homeDir(), 'downloads/apps.yaml'),
    filters: [{ name: 'apps', extensions: ['yaml', 'yml'] }],
  });
  if (pathToSave) {
    fs.writeTextFile(pathToSave, yaml.dump(apps));
  }
}
