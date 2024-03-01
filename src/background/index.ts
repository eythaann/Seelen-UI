import { loadBackgroundApi } from './api';
import { init } from './init';
import { getEnviroment } from './utils';
import { app, BrowserWindow, shell } from 'electron';
import isInstalling from 'electron-squirrel-startup';
import path from 'path';
import { updateElectronApp } from 'update-electron-app';

if (isInstalling) {
  app.quit();
}

if (getEnviroment() === 'installed') {
  updateElectronApp();
}

const createWindow = () => {
  const mainWindow = new BrowserWindow({
    width: 700,
    height: 500,
    maximizable: false,
    resizable: false,
    webPreferences: {
      preload: path.join(app.getAppPath(), 'dist/background-bundle/preload.js'),
      devTools: !app.isPackaged,
    },
    titleBarStyle: 'hidden',
    icon: path.join(app.getAppPath(), 'static/icons/icon.ico'),
    show: false,
  });

  mainWindow.webContents.setWindowOpenHandler((details) => {
    shell.openExternal(details.url); // Open URL in user's browser.
    return { action: 'deny' }; // Prevent the app from opening the URL.
  });

  mainWindow.loadFile(path.join(app.getAppPath(), 'dist/frontend-bundle/index.html'));

  mainWindow.once('ready-to-show', () => {
    mainWindow.show();
  });

  return mainWindow;
};

app.on('ready', () => {
  if (isInstalling || !app.requestSingleInstanceLock()) {
    return;
  }
  init();
  const mainWindow = createWindow();
  loadBackgroundApi(mainWindow);
});

app.on('window-all-closed', () => app.quit());
