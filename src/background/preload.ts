import { BackgroundApi, UserSettings } from '../shared.interfaces';

const { contextBridge, ipcRenderer } = require('electron');

const api: BackgroundApi = {
  enableAutostart: () => {
    ipcRenderer.send('enable-autostart');
  },
  disableAutostart: () => {
    ipcRenderer.send('disable-autostart');
  },
  autostartTaskExist: () => {
    return new Promise((resolve, reject) => {
      ipcRenderer.send('get-autostart-task');
      ipcRenderer.on('get-autostart-task-reply', (e, result, error) => {
        if (error) {
          return reject(error);
        }
        resolve(result != null);
      });
    });
  },
  getUserSettings: async () => {
    return new Promise<UserSettings>((resolve, reject) => {
      ipcRenderer.send('get-user-settings');
      ipcRenderer.on('get-user-settings-reply', (e, result: UserSettings, error) => {
        if (error) {
          return reject(error);
        }
        resolve(result);
      });
    });
  },
};

contextBridge.exposeInMainWorld('backgroundApi', api);