import { AppTemplate, defaultTheme, UserSettings } from '../../../../../shared.interfaces';
import { parseAsCamel, safeParseAsCamel, VariableConvention } from '../../../../shared/schemas';
import { Layout, LayoutSchema } from '../../../../shared/schemas/Layout';
import { Placeholder, PlaceholderSchema } from '../../../../shared/schemas/Placeholders';
import { AhkVariables, SettingsSchema } from '../../../../shared/schemas/Settings';
import { Theme, ThemeSchema } from '../../../../shared/schemas/Theme';
import { path } from '@tauri-apps/api';
import { invoke } from '@tauri-apps/api/core';
import { DirEntry } from '@tauri-apps/plugin-fs';
import yaml from 'js-yaml';
import { cloneDeep } from 'lodash';

import { resolveDotConfigPath } from '../config/infra';
import { dialog, fs } from '../tauri/infra';

import { AppsTemplates } from '../../../../shared/appsTemplates';

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

const isObject = (obj: any) => obj && typeof obj === 'object' && !Array.isArray(obj);
const mergeLayers = (obj1: any, obj2: any) => {
  const result = { ...obj1 };

  Object.keys(obj2).forEach((key) => {
    if (isObject(obj2[key])) {
      if (!obj1[key]) {
        result[key] = obj2[key];
      } else {
        result[key] = mergeLayers(obj1[key], obj2[key]);
      }
    } else {
      result[key] = obj1[key] !== undefined ? Math.max(obj1[key], obj2[key]) : obj2[key];
    }
  });

  return result;
};

export const getBackgroundLayers = (selected: string[], themes: Theme[]) => {
  return themes.reduce((acc, theme) => {
    if (selected.includes(theme.info.filename)) {
      return mergeLayers(acc, theme.layers);
    }

    return acc;
  }, cloneDeep(defaultTheme.layers));
};

async function loadUserThemes(ref: UserSettings) {
  async function themeFromDir(entry: Entry) {
    let theme = yaml.load(await fs.readTextFile(await path.join(entry.path, 'theme.yml'))) as Theme;
    theme = ThemeSchema.parse(theme) as Theme;

    theme.info.filename = entry.name;

    let wegPath = await path.join(entry.path, 'theme.weg.css');
    if (await fs.exists(wegPath)) {
      theme.styles.weg = await fs.readTextFile(wegPath);
    }

    let toolbarPath = await path.join(entry.path, 'theme.toolbar.css');
    if (await fs.exists(toolbarPath)) {
      theme.styles.toolbar = await fs.readTextFile(toolbarPath);
    }

    let wmPath = await path.join(entry.path, 'theme.wm.css');
    if (await fs.exists(wmPath)) {
      theme.styles.wm = await fs.readTextFile(wmPath);
    }

    return theme;
  }

  async function themeFromFile(entry: Entry) {
    let theme = yaml.load(await fs.readTextFile(entry.path)) as Theme;
    theme = ThemeSchema.parse(theme) as Theme;
    theme.info.filename = entry.name;
    return theme;
  }

  for (const entry of await getEntries('themes')) {
    let theme: null | Theme = null;

    if (entry.isDirectory) {
      theme = await themeFromDir(entry);
    } else if (entry.isFile && entry.name.endsWith('.yml')) {
      theme = await themeFromFile(entry);
    }

    if (theme) {
      ref.themes.push(theme);
    }
  }

  ref.bgLayers = getBackgroundLayers([ref.jsonSettings.selectedTheme].flat(), ref.themes);
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
  let found = false;

  for (const entry of await getEntries('placeholders')) {
    if (entry.isFile && entry.name.endsWith('.yml')) {
      let _placeholder = yaml.load(await fs.readTextFile(entry.path));

      let placeholder = safeParseAsCamel(PlaceholderSchema, _placeholder) as Placeholder;
      if (!placeholder) {
        continue;
      }

      placeholder.info.filename = entry.name;

      if (placeholder.info.displayName === 'Unknown') {
        placeholder.info.displayName = entry.name;
      }

      if (selectedPlaceholder === entry.name) {
        found = true;
      }

      ref.placeholders.push(placeholder);
    }
  }

  if (!found) {
    ref.jsonSettings.fancyToolbar.placeholder = ref.placeholders[0]?.info.filename || null;
  }
}

export async function loadUserSettings(route?: string): Promise<UserSettings> {
  const userSettings: UserSettings = {
    jsonSettings: parseAsCamel(SettingsSchema, {}),
    yamlSettings: [],
    themes: [],
    bgLayers: defaultTheme.layers,
    layouts: [],
    placeholders: [],
    env: await invoke('get_user_envs'),
  };

  const json_route = route || (await resolveDotConfigPath('settings.json'));
  const yaml_route = await resolveDotConfigPath('applications.yml');

  if (await fs.exists(json_route)) {
    userSettings.jsonSettings = parseAsCamel(
      SettingsSchema,
      JSON.parse(await fs.readTextFile(json_route)),
    );
  }

  if (await fs.exists(yaml_route)) {
    const processed = yaml.load(await fs.readTextFile(yaml_route));
    userSettings.yamlSettings = Array.isArray(processed) ? processed : [];
  }

  await loadUserThemes(userSettings);
  await loadUserLayouts(userSettings);
  await loadUserPlaceholders(userSettings);

  return userSettings;
}

export async function loadAppsTemplates() {
  const result: AppTemplate[] = [];

  for (const AppTemplateDeclaration of AppsTemplates) {
    const processed = yaml.load(
      await fs.readTextFile(
        await path.resolveResource(`static/apps_templates/${AppTemplateDeclaration.path}`),
      ),
    );
    const apps = Array.isArray(processed) ? processed : [];
    result.push({
      name: AppTemplateDeclaration.name,
      description: AppTemplateDeclaration.description,
      apps,
    });
  }

  return result;
}

export async function createAhkFiles(ahkVariables: AhkVariables) {
  const staticPath = await path.join(await path.resourceDir(), 'static');
  const entries = await fs.readDir(staticPath);

  for (const entry of entries) {
    if (entry.isFile && entry.name.endsWith('.ahk.template')) {
      let content = await fs.readTextFile(await path.join(staticPath, entry.name));
      content = content.replace(/{{(.*?)}}/g, (match, varname) => {
        return ahkVariables[varname]?.ahk || match;
      });
      await fs.writeTextFile(
        await path.join(staticPath, entry.name.replace('.template', '')),
        content,
      );
    }
  }
}

export async function saveUserSettings(settings: UserSettings) {
  const json_route = await resolveDotConfigPath('settings.json');
  const yaml_route = await resolveDotConfigPath('applications.yml');

  if (settings.jsonSettings.ahkEnabled) {
    await createAhkFiles(settings.jsonSettings.ahkVariables);
    invoke('start_seelen_shortcuts');
  } else {
    invoke('kill_seelen_shortcuts');
  }

  await fs.writeTextFile(
    json_route,
    JSON.stringify(VariableConvention.fromCamelToSnake(settings.jsonSettings)),
  );
  await fs.writeTextFile(yaml_route, yaml.dump(settings.yamlSettings));
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
