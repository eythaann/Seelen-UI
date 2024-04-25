import { AppTemplate, UserSettings } from '../../../../../shared.interfaces';
import { parseAsCamel, VariableConvention } from '../../../../utils/schemas';
import { SettingsSchema } from '../../../../utils/schemas/Settings';
import { Theme, ThemeSchema } from '../../../../utils/schemas/Theme';
import { path } from '@tauri-apps/api';
import { convertFileSrc, invoke } from '@tauri-apps/api/core';
import yaml from 'js-yaml';

import { dialog, fs } from '../tauri/infra';

import { AppsTemplates } from '../../../../utils/appsTemplates';

export async function loadUserSettings(route?: string): Promise<UserSettings> {
  const userSettings: UserSettings = {
    jsonSettings: parseAsCamel(SettingsSchema, {}),
    yamlSettings: [],
    themes: [],
    theme: null,
  };

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

      sanitizedTheme.info.filename = entry.name;

      const cssFilePath = await path.join(await path.resourceDir(), 'static', 'themes', entry.name.replace('.json', '.css'));
      if (await fs.exists(cssFilePath)) {
        sanitizedTheme.info.cssFileUrl = convertFileSrc(cssFilePath);
      }

      if (userSettings.jsonSettings.selectedTheme === entry.name) {
        userSettings.theme = sanitizedTheme;
      }

      userSettings.themes.push(sanitizedTheme);
    }
  }

  if (!userSettings.theme) {
    userSettings.theme = userSettings.themes[0] || null;
  }

  const json_route = route || await path.join(await path.homeDir(), '.config/seelen/settings.json');
  const yaml_route = await path.join(await path.homeDir(), '.config/seelen/applications.yml');

  if (await fs.exists(json_route)) {
    userSettings.jsonSettings = parseAsCamel(SettingsSchema, JSON.parse(await fs.readTextFile(json_route)));
  }

  if (await fs.exists(yaml_route)) {
    const processed = yaml.load(await fs.readTextFile(yaml_route));
    userSettings.yamlSettings = Array.isArray(processed) ? processed : [];
  }

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

export async function saveUserSettings(settings: Omit<UserSettings, 'themes' | 'theme'>) {
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
