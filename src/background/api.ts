import { StaticConfig } from '../JsonSettings.interface';
import { AppTemplate, UserSettings } from '../shared.interfaces';
import { ApplicationConfiguration } from '../YamlSettings.interface';
import { Channel, REPLY_BY_CHANNEL } from './constants';
import { execPrinter, fromPackageRoot, getEnviroment, runPwshScript } from './utils';
import { exec } from 'child_process';
import { app, BrowserWindow, dialog, ipcMain } from 'electron';
import { copyFileSync, existsSync, readFileSync, writeFileSync } from 'fs';
import { ensureFileSync, readJsonSync, writeJsonSync } from 'fs-extra';
import yaml from 'js-yaml';
import os from 'os';
import path from 'path';

import { AppsTemplates } from './appsTemplates';

const komorebi_config_path = path.join(os.homedir(), '.config/komorebi-ui');
const ahk_path = path.join(komorebi_config_path, '/komorebic.ahk');
const tryRunAhkShortcuts = () => {
  if (existsSync(ahk_path)) {
    exec(`"${ahk_path}"`, execPrinter);
  }
};

export const loadBackgroundApi = (mainWindow: BrowserWindow) => {
  ipcMain.on(Channel.ENABLE_AUTOSTART, (_event) => {
    switch (getEnviroment()) {
      case 'installed':
        return runPwshScript('autostart_on.ps1', `-VersionPath "${fromPackageRoot('../version')}"`);
      case 'packaged':
        return runPwshScript('autostart_on.ps1', `-ExeRoute "${fromPackageRoot('/komorebi.exe')}"`);
      default:
        return;
    }
  });

  ipcMain.on(Channel.DISABLE_AUTOSTART, (_event) => {
    runPwshScript('autostart_off.ps1');
  });

  ipcMain.on(Channel.GET_AUTOSTART_STATUS, (event) => {
    const command = 'schtasks /query /tn "KomorebiUI" /v';
    exec(command, (err, stdout, stderr) => {
      if (stderr) {
        event.sender.send(REPLY_BY_CHANNEL[Channel.GET_AUTOSTART_STATUS], null);
        return;
      }
      event.sender.send(REPLY_BY_CHANNEL[Channel.GET_AUTOSTART_STATUS], stdout);
    });
  });

  ipcMain.on(Channel.GET_USER_SETTINGS, (event, route?: string) => {
    const json_route = route || path.join(os.homedir(), '.config/komorebi-ui/settings.json');
    let data_json: StaticConfig = {};
    let data_yaml: ApplicationConfiguration[] = [];

    if (existsSync(json_route)) {
      data_json = readJsonSync(json_route);

      let pathToYml = data_json.app_specific_configuration_path;
      if (pathToYml) {
        pathToYml = pathToYml.replace('$Env:USERPROFILE', '~');
        if (pathToYml.startsWith('~')) {
          pathToYml = path.join(os.homedir(), pathToYml.slice(2));
        }

        if (existsSync(pathToYml)) {
          const processed = yaml.load(readFileSync(pathToYml, 'utf-8'));
          data_yaml = Array.isArray(processed) ? processed : [];
        }
      }
    }

    const userSettings: UserSettings = {
      jsonSettings: data_json,
      yamlSettings: data_yaml,
      ahkEnabled: existsSync(ahk_path) && !!data_json.ahk_enabled,
    };

    event.sender.send(REPLY_BY_CHANNEL[Channel.GET_USER_SETTINGS], userSettings);
  });

  ipcMain.on(Channel.SAVE_USER_SETTINGS, (event, settings: UserSettings) => {
    const json_route = path.join(os.homedir(), '.config/komorebi-ui/settings.json');
    const yaml_route = path.join(os.homedir(), '.config/komorebi-ui/applications.yml');
    settings.jsonSettings.app_specific_configuration_path = yaml_route;

    let oldSettings: StaticConfig = {};
    if (existsSync(json_route)) {
      oldSettings = readJsonSync(json_route);
    }

    const isChangingAhkStatus = settings.ahkEnabled != oldSettings.ahk_enabled;
    if (settings.ahkEnabled) {
      if (!existsSync(ahk_path)) {
        copyFileSync(path.join(app.getAppPath(), 'komorebi.sample.ahk'), ahk_path);
        copyFileSync(
          path.join(app.getAppPath(), 'komorebic.lib.ahk'),
          path.join(komorebi_config_path, '/komorebic.lib.ahk'),
        );
      }
      if (isChangingAhkStatus) {
        tryRunAhkShortcuts();
      }
    } else if (isChangingAhkStatus) {
      exec('wmic process where "commandline like \'%komorebic.ahk%\'" call terminate', { shell: 'powershell.exe' }, execPrinter);
    }

    settings.jsonSettings.ahk_enabled = settings.ahkEnabled;
    ensureFileSync(json_route);
    writeJsonSync(json_route, settings.jsonSettings);
    writeFileSync(yaml_route, yaml.dump(settings.yamlSettings));

    event.sender.send(REPLY_BY_CHANNEL[Channel.SAVE_USER_SETTINGS]);
  });

  ipcMain.on(Channel.IMPORT_APPS, (event) => {
    const defaultPath = app.isPackaged
      ? fromPackageRoot('./resources/apps_templates')
      : path.join(app.getAppPath(), 'static/apps_templates');

    dialog
      .showOpenDialog(mainWindow, {
        defaultPath,
        properties: ['openFile', 'multiSelections'],
        buttonLabel: 'load',
        title: 'Select template',
        filters: [{ name: 'apps', extensions: ['yaml', 'yml'] }],
      })
      .then((result) => {
        if (result.canceled) {
          return;
        }
        const data: ApplicationConfiguration[] = result.filePaths.flatMap((path) => {
          const processed = yaml.load(readFileSync(path, 'utf-8'));
          return Array.isArray(processed) ? processed : [];
        });
        event.sender.send(REPLY_BY_CHANNEL[Channel.IMPORT_APPS], data);
      })
      .catch((err) => {
        event.sender.send(REPLY_BY_CHANNEL[Channel.IMPORT_APPS], undefined, err);
      });
  });

  ipcMain.on(Channel.LOAD_APPS_TEMPLATES, (event) => {
    const defaultPath = app.isPackaged
      ? fromPackageRoot('./resources/apps_templates')
      : path.join(app.getAppPath(), 'static/apps_templates');

    const result = AppsTemplates.map<AppTemplate>(({ name, description, path: file }) => {
      const apps = yaml.load(readFileSync(path.join(defaultPath, file), 'utf-8')) as ApplicationConfiguration[];
      return {
        name,
        description,
        apps,
      };
    });

    event.sender.send(REPLY_BY_CHANNEL[Channel.LOAD_APPS_TEMPLATES], result);
  });

  ipcMain.on(Channel.EXPORT_APPS, (event, apps: ApplicationConfiguration[]) => {
    const pathToSave = dialog.showSaveDialogSync(mainWindow, {
      title: 'Exporting Apps',
      defaultPath: path.join(os.homedir(), 'downloads/apps.yaml'),
      filters: [{ name: 'apps', extensions: ['yaml', 'yml'] }],
    });
    if (pathToSave) {
      writeFileSync(pathToSave, yaml.dump(apps));
    }
    event.sender.send(REPLY_BY_CHANNEL[Channel.EXPORT_APPS]);
  });

  ipcMain.on(Channel.QUIT, () => {
    mainWindow.close();
  });

  ipcMain.on(Channel.AHK_SETUP, () => {
    exec(`"${fromPackageRoot('resources/redis/AutoHotKey_setup.exe')}"`, execPrinter);
  });

  ipcMain.on(Channel.RESTART, async () => {
    await runPwshScript('force_stop.ps1', `-ExeRoute "${fromPackageRoot('/komorebi.exe')}"`);
    await runPwshScript('manual_run.ps1', `-ExeRoute "${fromPackageRoot('/komorebi.exe')}"`);
  });
};
