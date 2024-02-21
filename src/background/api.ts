import { StaticConfig } from '../JsonSettings.interface';
import { UserSettings } from '../shared.interfaces';
import { Channel, REPLY_BY_CHANNEL } from './constants';
import { fromPackageRoot, runPwshScript } from './utils';
import { exec } from 'child_process';
import { ipcMain } from 'electron';
import { existsSync, readFileSync, writeFileSync } from 'fs';
import yaml from 'js-yaml';
import os from 'os';
import path from 'path';

export const loadApi = () => {
  ipcMain
    .on('enable-autostart', (_event) => {
      runPwshScript('autostart_on.ps1', `-ExeRoute "${fromPackageRoot('/komorebi.exe')}"`);
    })

    .on('disable-autostart', (_event) => {
      runPwshScript('autostart_off.ps1');
    })

    .on('get-autostart-task', (event) => {
      const command = 'schtasks /query /tn "KomorebiUI" /v';
      exec(command, (err, stdout, stderr) => {
        if (stderr) {
          event.sender.send('get-autostart-task-reply', null);
          return;
        }
        event.sender.send('get-autostart-task-reply', stdout);
      });
    })

    .on('get-user-settings', (event) => {
      const json_route = path.join(os.homedir(), '.config/komorebi/settings.json');
      let data_json = {} as StaticConfig;
      let data_yaml = [];

      if (existsSync(json_route)) {
        data_json = JSON.parse(readFileSync(json_route, 'utf-8'));
        let pathToYml = data_json.app_specific_configuration_path;
        if (pathToYml) {
          if (pathToYml.startsWith('~')) {
            pathToYml = path.join(os.homedir(), pathToYml.slice(2));
          }
          const processed = yaml.load(readFileSync(pathToYml, 'utf-8'));
          data_yaml = Array.isArray(processed) ? processed : [];
        }
      }

      event.sender.send('get-user-settings-reply', {
        jsonSettings: data_json,
        yamlSettings: data_yaml,
      });
    })

    .on(Channel.SAVE_USER_SETTINGS, (event, settings: UserSettings) => {
      const json_route = path.join(os.homedir(), '.config/komorebi/settings.json');
      writeFileSync(json_route, JSON.stringify(settings.jsonSettings));
      event.sender.send(REPLY_BY_CHANNEL[Channel.SAVE_USER_SETTINGS]);
    });
};
