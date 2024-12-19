import { disableWebviewShortcutsAndContextMenu, getCurrentWidget } from '@seelen-ui/lib';
import * as Logger from '@tauri-apps/plugin-log';

export function wrapConsole() {
  window.addEventListener('unhandledrejection', (event) => {
    console.error(event.reason);
    Logger.error(`Unhandled Rejection - ${event.reason}`);
  });

  const WebConsole = {
    info: console.info,
    warn: console.warn,
    error: console.error,
    debug: console.debug,
    trace: console.trace,
  };

  const widget = getCurrentWidget();
  Logger.info(`Registering ${widget.label} webview console as logger`);
  const StringifyParams = (params: any[]): string => {
    return (
      `[${widget.id}]: ` +
      params.reduce((a, b) => {
        if (typeof b === 'string') {
          return a + ' ' + b;
        }
        return a + ' ' + JSON.stringify(b, null, 2);
      }, '')
    );
  };

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

  disableWebviewShortcutsAndContextMenu();
}
