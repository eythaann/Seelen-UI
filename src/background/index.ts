import { loadBackgroundApi } from './api';
import { fromPackageRoot, runPwshScript } from './utils';
import { app, BrowserWindow } from 'electron';
import { shell } from 'electron';
import { copyFileSync, existsSync } from 'fs';
import path from 'path';
import { updateElectronApp } from 'update-electron-app';

if (require('electron-squirrel-startup')) {
  app.quit();
}

updateElectronApp();

app.on('ready', () => {
  if (app.isPackaged) {
    if (!existsSync(fromPackageRoot('/komorebi.exe'))) {
      copyFileSync(path.join(app.getAppPath(), 'komorebi.exe'), fromPackageRoot('/komorebi.exe'));
      copyFileSync(path.join(app.getAppPath(), 'komorebic.exe'), fromPackageRoot('/komorebic.exe'));
    }

    runPwshScript('add_to_path.ps1', `-AppPath "${fromPackageRoot()}\\"`);
    runPwshScript('manual_run.ps1', `-ExeRoute "${fromPackageRoot('/komorebi.exe')}"`);
  }

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
  });

  mainWindow.webContents.setWindowOpenHandler((details) => {
    shell.openExternal(details.url); // Open URL in user's browser.
    return { action: 'deny' }; // Prevent the app from opening the URL.
  });

  mainWindow.loadFile(path.join(app.getAppPath(), 'dist/frontend-bundle/index.html'));

  loadBackgroundApi(mainWindow);
});

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit();
  };
});