const { app, BrowserWindow, ipcMain } = require('electron');
const path = require('path');
const { shell } = require('electron');
const { exec } = require('child_process');
const { runPwshScript, fromPackage } = require('./utils');

// Handle creating/removing shortcuts on Windows when installing/uninstalling.
if (require('electron-squirrel-startup')) {
  app.quit();
}

const createWindow = () => {
  if (app.isPackaged) {
    runPwshScript('manual_run.ps1', `-ExeRoute "${fromPackage('/komorebi.exe')}"`);
  }

  const mainWindow = new BrowserWindow({
    width: 700,
    height: 500,
    maximizable: false,
    resizable: false,
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
      devTools: !app.isPackaged,
    },
    titleBarStyle: 'hidden',
    icon: path.join(__dirname, 'assets/icons/icon.ico'),
  });

  mainWindow.webContents.setWindowOpenHandler((details) => {
    shell.openExternal(details.url); // Open URL in user's browser.
    return { action: 'deny' }; // Prevent the app from opening the URL.
  });

  mainWindow.loadFile(path.join(__dirname, '../dist/frontend-bundle/index.html'));

  ipcMain.on('enable-autostart', (_event, _arg) => {
    runPwshScript('autostart_on.ps1');
  });

  ipcMain.on('disable-autostart', (_event, _arg) => {
    runPwshScript('autostart_off.ps1');
  });

  ipcMain.on('get-autostart-task', (event, _arg) => {
    const command = 'schtasks /query /tn "KomorebiUI" /v';
    exec(command, (er, stdout, stderr) => {
      if (stderr) {
        event.sender.send('get-autostart-task-reply', null);
        return;
      }
      event.sender.send('get-autostart-task-reply', stdout);
    });
  });
};

app.on('ready', createWindow);