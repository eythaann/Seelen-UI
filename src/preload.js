const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('backgroundApi', {
  enableAutostart: () => {
    ipcRenderer.send('enable-autostart');
  },
  disableAutostart: () => {
    ipcRenderer.send('disable-autostart');
  },
  autostartTaskExist: () => {},
});