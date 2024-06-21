import * as Logger from '@tauri-apps/plugin-log';
import { exit } from '@tauri-apps/plugin-process';

declare global {
  interface Console {
    throw(...params: any[]): never;
  }
}

export function wrapConsole() {
  const WebConsole = {
    info: console.info,
    warn: console.warn,
    error: console.error,
    debug: console.debug,
    trace: console.trace,
  };

  const StrintifyParams = (params: any[]): string => {
    return params.reduce((a, b) => {
      if (typeof b === 'string') {
        return a + ' ' + b;
      }
      return a + ' ' + JSON.stringify(b, null, 2);
    }, '');
  };

  window.addEventListener('unhandledrejection', (event) => {
    console.error(`Unhandled Rejection - ${event.reason}`);
    exit(1);
  });

  console.error = (...params: any[]) => {
    WebConsole.error(...params);
    Logger.error(StrintifyParams(params));
  };

  console.throw = (...params: any[]) => {
    console.error(...params);
    throw new Error();
  };

  console.warn = (...params: any[]) => {
    WebConsole.warn(...params);
    Logger.warn(StrintifyParams(params));
  };

  console.info = (...params: any[]) => {
    WebConsole.info(...params);
    Logger.info(StrintifyParams(params));
  };

  console.debug = (...params: any[]) => {
    WebConsole.debug(...params);
    Logger.debug(StrintifyParams(params));
  };

  console.trace = (...params: any[]) => {
    WebConsole.trace(...params);
    Logger.trace(StrintifyParams(params));
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
