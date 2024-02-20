import { fromPackageRoot, runPwshScript } from './utils';
import { exec } from 'child_process';
import { ipcMain } from 'electron';

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
    });
};