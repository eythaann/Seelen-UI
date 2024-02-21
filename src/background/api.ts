import { StaticConfig } from '../JsonSettings.interface';
import { UserSettings } from '../shared.interfaces';
import { ApplicationConfiguration } from '../YamlSettings.interface';
import { Channel, REPLY_BY_CHANNEL } from './constants';
import { fromPackageRoot, runPwshScript } from './utils';
import { exec } from 'child_process';
import { ipcMain } from 'electron';
import { existsSync, readFileSync, writeFileSync } from 'fs';
import yaml from 'js-yaml';
import os from 'os';
import path from 'path';

export const loadBackgroundApi = () => {
  ipcMain
    .on(Channel.ENABLE_AUTOSTART, (_event) => {
      runPwshScript('autostart_on.ps1', `-ExeRoute "${fromPackageRoot('/komorebi.exe')}"`);
    })

    .on(Channel.DISABLE_AUTOSTART, (_event) => {
      runPwshScript('autostart_off.ps1');
    })

    .on(Channel.GET_AUTOSTART_STATUS, (event) => {
      const command = 'schtasks /query /tn "KomorebiUI" /v';
      exec(command, (err, stdout, stderr) => {
        if (stderr) {
          event.sender.send(REPLY_BY_CHANNEL[Channel.GET_AUTOSTART_STATUS], null);
          return;
        }
        event.sender.send(REPLY_BY_CHANNEL[Channel.GET_AUTOSTART_STATUS], stdout);
      });
    })

    .on(Channel.GET_USER_SETTINGS, (event) => {
      const json_route = path.join(os.homedir(), '.config/komorebi/settings.json');
      let data_json: StaticConfig = {};
      let data_yaml: ApplicationConfiguration[] = [];

      if (existsSync(json_route)) {
        data_json = JSON.parse(readFileSync(json_route, 'utf-8'));

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

      event.sender.send(REPLY_BY_CHANNEL[Channel.GET_USER_SETTINGS], {
        jsonSettings: data_json,
        yamlSettings: data_yaml,
      });
    })

    .on(Channel.SAVE_USER_SETTINGS, (event, settings: UserSettings) => {
      const json_route = path.join(os.homedir(), '.config/komorebi/settings.json');
      const yaml_route = path.join(os.homedir(), '.config/komorebi/applications.yml');

      settings.jsonSettings.app_specific_configuration_path = yaml_route;

      writeFileSync(json_route, JSON.stringify(settings.jsonSettings));
      writeFileSync(yaml_route, yaml.dump(settings.yamlSettings));

      event.sender.send(REPLY_BY_CHANNEL[Channel.SAVE_USER_SETTINGS]);
    });
};
