const { app, BrowserWindow, ipcMain } = require('electron');
const path = require('path');
const { shell } = require('electron');
const { exec } = require('child_process');

// Handle creating/removing shortcuts on Windows when installing/uninstalling.
if (require('electron-squirrel-startup')) {
  app.quit();
}

const execPrinter = (error, stdout, stderr) => {
  if (error) {
    console.error(`Error: ${error.message}`);
  }
  if (stderr) {
    console.error(`STDERR: ${stderr}`);
  }
  if (stdout) {
    console.log(`STDOUT: ${stdout}`);
  }
};

const createWindow = () => {
  const mainWindow = new BrowserWindow({
    width: 700,
    height: 500,
    maximizable: false,
    resizable: false,
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
      //devTools: !app.isPackaged,
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
    const command = `powershell -Command "Start-Process powershell -Verb RunAs -ArgumentList '-NoProfile -ExecutionPolicy Bypass -File "${path.join(__dirname, './autostart_on.ps1')}" -WindowStyle Hidden'" -WindowStyle Hidden`;
    exec(command, execPrinter);
  });

  ipcMain.on('disable-autostart', (_event, _arg) => {
    const command = `powershell -Command "Start-Process powershell -Verb RunAs -ArgumentList '-NoProfile -ExecutionPolicy Bypass -File "${path.join(__dirname, './autostart_off.ps1')}" -WindowStyle Hidden'" -WindowStyle Hidden`;
    exec(command, execPrinter);
  });
};

// This method will be called when Electron has finished
// initialization and is ready to create browser windows.
// Some APIs can only be used after this event occurs.
app.on('ready', createWindow);

// Quit when all windows are closed, except on macOS. There, it's common
// for applications and their menu bar to stay active until the user quits
// explicitly with Cmd + Q.
app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit();
  }
});

app.on('activate', () => {
  // On OS X it's common to re-create a window in the app when the
  // dock icon is clicked and there are no other windows open.
  if (BrowserWindow.getAllWindows().length === 0) {
    createWindow();
  }
});