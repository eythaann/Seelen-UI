import { AppTemplate, BackgroundApi, UserSettings } from '../shared.interfaces';
import { ApplicationConfiguration } from '../YamlSettings.interface';
import { Channel, REPLY_BY_CHANNEL } from './constants';

const { contextBridge, ipcRenderer } = require('electron');

const api: BackgroundApi = {
  enableAutostart() {
    ipcRenderer.send(Channel.ENABLE_AUTOSTART);
  },
  disableAutostart() {
    ipcRenderer.send(Channel.DISABLE_AUTOSTART);
  },
  quit() {
    ipcRenderer.send(Channel.QUIT);
  },
  forceRestart() {
    ipcRenderer.send(Channel.RESTART);
  },
  autostartTaskExist() {
    return new Promise((resolve, reject) => {
      ipcRenderer.send(Channel.GET_AUTOSTART_STATUS);
      ipcRenderer.on(REPLY_BY_CHANNEL[Channel.GET_AUTOSTART_STATUS], (e, result, error) => {
        if (error) {
          return reject(error);
        }
        resolve(result != null);
      });
    });
  },
  getUserSettings(route?: string) {
    return new Promise<UserSettings>((resolve, reject) => {
      ipcRenderer.send(Channel.GET_USER_SETTINGS, route);
      ipcRenderer.on(REPLY_BY_CHANNEL[Channel.GET_USER_SETTINGS], (e, result: UserSettings, error) => {
        if (error) {
          return reject(error);
        }
        resolve(result);
      });
    });
  },
  saveUserSettings(settings) {
    return new Promise<void>((resolve, reject) => {
      ipcRenderer.send(Channel.SAVE_USER_SETTINGS, settings);
      ipcRenderer.on(REPLY_BY_CHANNEL[Channel.SAVE_USER_SETTINGS], (e, result, error) => {
        if (error) {
          return reject(error);
        }
        resolve();
      });
    });
  },
  importApps() {
    return new Promise<ApplicationConfiguration[]>((resolve, reject) => {
      ipcRenderer.send(Channel.IMPORT_APPS);
      ipcRenderer.on(REPLY_BY_CHANNEL[Channel.IMPORT_APPS], (e, result: ApplicationConfiguration[], error) => {
        if (error) {
          return reject(error);
        }
        resolve(result);
      });
    });
  },
  loadAppsTemplates() {
    return new Promise<AppTemplate[]>((resolve, reject) => {
      ipcRenderer.send(Channel.LOAD_APPS_TEMPLATES);
      ipcRenderer.on(REPLY_BY_CHANNEL[Channel.LOAD_APPS_TEMPLATES], (e, result: AppTemplate[], error) => {
        if (error) {
          return reject(error);
        }
        resolve(result);
      });
    });
  },
  exportApps(apps) {
    return new Promise((resolve) => {
      ipcRenderer.send(Channel.EXPORT_APPS, apps);
      ipcRenderer.on(REPLY_BY_CHANNEL[Channel.EXPORT_APPS], () => resolve());
    });
  },
  // installers
  runAhkSetup() {
    ipcRenderer.send(Channel.AHK_SETUP);
  },
};

contextBridge.exposeInMainWorld('backgroundApi', api);