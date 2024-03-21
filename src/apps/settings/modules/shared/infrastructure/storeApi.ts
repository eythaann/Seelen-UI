import { AppTemplate, UserSettings } from '../../../../../shared.interfaces';
import { ApplicationConfiguration } from '../../../../../YamlSettings.interface';
import { dialog, fs } from './tauri';
import { path } from '@tauri-apps/api';
import yaml from 'js-yaml';

import { AppsTemplates } from '../domain/appsTemplates';

export async function loadUserSettings(route?: string): Promise<UserSettings> {
  const userSettings: UserSettings = {
    jsonSettings: {},
    yamlSettings: [],
    ahkEnabled: false,
    updateNotification: false,
  };

  const json_route =
    route || (await path.join(await path.homeDir(), '.config/komorebi-ui/settings.json'));

  if (!(await fs.exists(json_route))) {
    return userSettings;
  }

  userSettings.jsonSettings = JSON.parse(await fs.readTextFile(json_route));

  let pathToYml = userSettings.jsonSettings.app_specific_configuration_path;
  if (pathToYml) {
    pathToYml = pathToYml.replace('$Env:USERPROFILE', '~');
    if (pathToYml.startsWith('~')) {
      pathToYml = await path.join(await path.homeDir(), pathToYml.slice(2));
    }

    const processed = yaml.load(await fs.readTextFile(pathToYml));
    userSettings.yamlSettings = Array.isArray(processed) ? processed : [];
  }

  userSettings.ahkEnabled = !!userSettings.jsonSettings.ahk_enabled;
  userSettings.updateNotification = !!userSettings.jsonSettings.update_notification;
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
  const json_route = await path.join(await path.homeDir(), '.config/komorebi-ui/settings.json');
  const yaml_route = await path.join(await path.homeDir(), '.config/komorebi-ui/applications.yml');
  settings.jsonSettings.app_specific_configuration_path = yaml_route;

  /*   let oldSettings: StaticConfig = {};
  if (await fs.exists(json_route)) {
    oldSettings = JSON.parse(await fs.readTextFile(json_route));
  } */

  /*   const isChangingAhkStatus = settings.ahkEnabled != oldSettings.ahk_enabled;
  if (settings.ahkEnabled) {
    if (!await fs.exists(ahk_path)) {
      await fs.copyFile(await path.join(app.getAppPath(), 'komorebi.sample.ahk'), ahk_path);
      await fs.copyFile(
        await path.join(app.getAppPath(), 'komorebic.lib.ahk'),
        await path.join(komorebi_config_path, '/komorebic.lib.ahk'),
      );
    }
    if (isChangingAhkStatus) {
      tryRunAhkShortcuts();
    }
  } else if (isChangingAhkStatus) {
    exec('wmic process where "commandline like \'%komorebic.ahk%\'" call terminate', { shell: 'powershell.exe' }, execPrinter);
  } */

  settings.jsonSettings.ahk_enabled = settings.ahkEnabled;
  settings.jsonSettings.update_notification = settings.updateNotification;

  if (!(await fs.exists(json_route))) {
    await fs.mkdir(await path.join(await path.homeDir(), '.config'));
    await fs.mkdir(await path.join(await path.homeDir(), '.config/komorebi-ui'));
  }

  await fs.writeTextFile(json_route, JSON.stringify(settings.jsonSettings));
  await fs.writeTextFile(yaml_route, yaml.dump(settings.yamlSettings));
}

export async function ImportApps() {
  const data: ApplicationConfiguration[] = [];

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

export async function ExportApps(apps: ApplicationConfiguration[]) {
  const pathToSave = await dialog.save({
    title: 'Exporting Apps',
    defaultPath: await path.join(await path.homeDir(), 'downloads/apps.yaml'),
    filters: [{ name: 'apps', extensions: ['yaml', 'yml'] }],
  });
  if (pathToSave) {
    fs.writeTextFile(pathToSave, yaml.dump(apps));
  }
}
