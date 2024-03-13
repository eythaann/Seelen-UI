import * as Logger from '@tauri-apps/plugin-log';

export function wrapConsole() {
  const WebConsole = {
    info: console.info,
    warn: console.warn,
    error: console.error,
    debug: console.debug,
    trace: console.trace,
  };

  window.addEventListener('unhandledrejection', (event) => {
    console.error(`Unhandled Rejection - ${event.reason}`);
  });

  console.error = (message: any, ...optionalParams: any[]) => {
    WebConsole.error(message, ...optionalParams);
    Logger.error(String(message));
  };

  console.warn = (message: any, ...optionalParams: any[]) => {
    WebConsole.warn(message, ...optionalParams);
    Logger.warn(String(message));
  };

  console.info = (message: any, ...optionalParams: any[]) => {
    WebConsole.info(message, ...optionalParams);
    Logger.info(String(message));
  };

  console.debug = (message: any, ...optionalParams: any[]) => {
    WebConsole.debug(message, ...optionalParams);
    Logger.debug(String(message));
  };

  console.trace = (message: any, ...optionalParams: any[]) => {
    WebConsole.trace(message, ...optionalParams);
    Logger.trace(String(message));
  };
}