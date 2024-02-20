import { fromPackageRoot, runPwshScript } from './utils';
import { exec } from 'child_process';
import { ipcMain } from 'electron';
import { existsSync, readFileSync } from 'fs';
import os from 'os';
import path from 'path';

export const loadApi = () => {
  ipcMain
    .on('enable-autostart', (_event, _arg) => {
      runPwshScript('autostart_on.ps1', `-ExeRoute "${fromPackageRoot('/komorebi.exe')}"`);
    })

    .on('disable-autostart', (_event, _arg) => {
      runPwshScript('autostart_off.ps1');
    })

    .on('get-autostart-task', (event, _arg) => {
      const command = 'schtasks /query /tn "KomorebiUI" /v';
      exec(command, (err, stdout, stderr) => {
        if (stderr) {
          event.sender.send('get-autostart-task-reply', null);
          return;
        }
        event.sender.send('get-autostart-task-reply', stdout);
      });
    })

    .on('get-user-settings', (event, _arg) => {
      const json_route = path.join(os.homedir(), '.config/komorebi/settings.json');
      let data_json = {};
      if (existsSync(json_route)) {
        data_json = JSON.parse(readFileSync(json_route, 'utf-8'));
      }
      event.sender.send('get-user-settings-reply', {
        jsonSettings: data_json,
        yamlSettings: [],
      });
    });
};