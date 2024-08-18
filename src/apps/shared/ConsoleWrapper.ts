import { getCurrentWebview } from '@tauri-apps/api/webview';
import * as Logger from '@tauri-apps/plugin-log';

export function wrapConsole() {
  const WebConsole = {
    info: console.info,
    warn: console.warn,
    error: console.error,
    debug: console.debug,
    trace: console.trace,
  };

  const label = getCurrentWebview().label;
  const StringifyParams = (params: any[]): string => {
    return label + ':' + params.reduce((a, b) => {
      if (typeof b === 'string') {
        return a + ' ' + b;
      }
      return a + ' ' + JSON.stringify(b, null, 2);
    }, '');
  };

  window.addEventListener('unhandledrejection', (event) => {
    console.error(`Unhandled Rejection - ${event.reason}`);
  });

  console.error = (...params: any[]) => {
    WebConsole.error(...params);
    Logger.error(StringifyParams(params));
  };

  console.warn = (...params: any[]) => {
    WebConsole.warn(...params);
    Logger.warn(StringifyParams(params));
  };

  console.info = (...params: any[]) => {
    WebConsole.info(...params);
    Logger.info(StringifyParams(params));
  };

  console.debug = (...params: any[]) => {
    WebConsole.debug(...params);
    Logger.debug(StringifyParams(params));
  };

  console.trace = (...params: any[]) => {
    WebConsole.trace(...params);
    Logger.trace(StringifyParams(params));
  };

  disableRefreshAndContextMenu();
}

export function disableRefreshAndContextMenu() {
  document.addEventListener('keydown', function (event) {
    if (
      event.key === 'F5' ||
      (event.ctrlKey && event.key === 'r') ||
      (event.metaKey && event.key === 'r')
    ) {
      event.preventDefault();
    }
  });

  document.addEventListener('contextmenu', function (event) {
    event.preventDefault();
  });
}
