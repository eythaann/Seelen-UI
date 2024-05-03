import { AppTemplate, UserSettings } from '../../../../../shared.interfaces';
import { parseAsCamel, VariableConvention } from '../../../../utils/schemas';
import { Layout, LayoutSchema } from '../../../../utils/schemas/Layout';
import { Placeholder, PlaceholderSchema } from '../../../../utils/schemas/Placeholders';
import { SettingsSchema } from '../../../../utils/schemas/Settings';
import { Theme, ThemeSchema } from '../../../../utils/schemas/Theme';
import { path } from '@tauri-apps/api';
import { convertFileSrc, invoke } from '@tauri-apps/api/core';
import yaml from 'js-yaml';

import { dialog, fs } from '../tauri/infra';

import { AppsTemplates } from '../../../../utils/appsTemplates';

async function loadUserThemes(ref: UserSettings) {
  const themesPath = await path.join(await path.resourceDir(), 'static', 'themes');
  const entries = await fs.readDir(themesPath);

  for (const entry of entries) {
    if (entry.isFile && entry.name.endsWith('.json')) {
      let theme: Theme = JSON.parse(await fs.readTextFile(await path.join(themesPath, entry.name)));
      theme = parseAsCamel(ThemeSchema, theme);

      const sanitizedTheme: Theme = {
        ...theme,
        info: {
          ...theme.info,
          filename: entry.name,
          cssFileUrl: null,
        },
      };

      if (sanitizedTheme.info.displayName === 'Unknown') {
        sanitizedTheme.info.displayName = entry.name;
      }

      const cssFilePath = await path.join(await path.resourceDir(), 'static', 'themes', entry.name.replace('.json', '.css'));
      if (await fs.exists(cssFilePath)) {
        sanitizedTheme.info.cssFileUrl = convertFileSrc(cssFilePath);
      }

      if (ref.jsonSettings.selectedTheme === entry.name) {
        ref.theme = sanitizedTheme;
      }

      ref.themes.push(sanitizedTheme);
    }
  }

  if (!ref.theme) {
    ref.theme = ref.themes[0] || null;
    ref.jsonSettings.selectedTheme = ref.theme?.info.filename || null;
  }
}

async function loadUserLayouts(ref: UserSettings) {
  const layoutsPath = await path.join(await path.resourceDir(), 'static', 'layouts');
  const entries = await fs.readDir(layoutsPath);

  const defaultLayout = ref.jsonSettings.windowManager.defaultLayout;
  let found = false;

  for (const entry of entries) {
    if (entry.isFile && entry.name.endsWith('.json')) {
      let layout: Layout = JSON.parse(await fs.readTextFile(await path.join(layoutsPath, entry.name)));
      layout = parseAsCamel(LayoutSchema, layout);

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
  const placeholderPath = await path.join(await path.resourceDir(), 'static', 'placeholders');
  const entries = await fs.readDir(placeholderPath);

  const selectedPlaceholder = ref.jsonSettings.fancyToolbar.placeholder;
  let found = false;

  for (const entry of entries) {
    if (entry.isFile && entry.name.endsWith('.yml')) {
      let _placeholder = yaml.load(await fs.readTextFile(await path.join(placeholderPath, entry.name)));
      let placeholder = parseAsCamel(PlaceholderSchema, _placeholder) as Placeholder;

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
    theme: null,
    layouts: [],
    placeholders: [],
    env: await invoke('get_user_envs'),
  };

  const json_route = route || await path.join(await path.homeDir(), '.config/seelen/settings.json');
  const yaml_route = await path.join(await path.homeDir(), '.config/seelen/applications.yml');

  if (await fs.exists(json_route)) {
    userSettings.jsonSettings = parseAsCamel(SettingsSchema, JSON.parse(await fs.readTextFile(json_route)));
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

export async function saveUserSettings(settings: UserSettings) {
  const json_route = await path.join(await path.homeDir(), '.config/seelen/settings.json');
  const yaml_route = await path.join(await path.homeDir(), '.config/seelen/applications.yml');

  if (settings.jsonSettings.ahkEnabled) {
    invoke('start_seelen_shortcuts');
  } else {
    invoke('kill_seelen_shortcuts');
  }

  await fs.writeTextFile(json_route, JSON.stringify(VariableConvention.fromCamelToSnake(settings.jsonSettings)));
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
