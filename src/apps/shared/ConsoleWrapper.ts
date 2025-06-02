import { logger } from '@seelen-ui/lib/tauri';

function StringifyParams(params: any[]): string {
  return params.reduce((acc, current) => {
    if (typeof current === 'string') {
      return acc + ' ' + current;
    }

    if (current instanceof Error) {
      return `${acc} ${current.name}(${current.message})\n${current.stack}`;
    }

    if (typeof current === 'object') {
      let stringObj = '';
      try {
        stringObj = JSON.stringify(current, null, 2);
      } catch (_e) {
        stringObj = `${current}`;
      }
      return acc + ' ' + stringObj;
    }

    return acc + ' ' + `${current}`;
  }, '');
};

function decodeUrlSafeBase64(base64Str: string) {
  let standardBase64 = base64Str.replace(/-/g, '+').replace(/_/g, '/');
  const padLength = (4 - (standardBase64.length % 4)) % 4;
  standardBase64 += '='.repeat(padLength);
  return atob(standardBase64);
}

class WebviewInformation {
  _label: string | null = null;
  get label() {
    if (this._label) {
      return this._label;
    }

    const viewLabel = window.__TAURI_INTERNALS__?.metadata?.currentWebview?.label;
    this._label = viewLabel ? decodeUrlSafeBase64(viewLabel) : 'Unknown';
    return this._label;
  }
};

export function wrapConsoleV2() {
  const WebConsole = {
    info: console.info,
    warn: console.warn,
    error: console.error,
    debug: console.debug,
    trace: console.trace,
  };

  function forwardConsole(
    fnName: keyof typeof WebConsole,
    logger: (message: string) => Promise<void>,
  ) {
    const original = console[fnName];
    console[fnName] = (...params: any[]) => {
      original(...params);
      logger(`[${new WebviewInformation().label}]: ` + StringifyParams(params));
    };
  }

  // forwardConsole('log', trace);
  forwardConsole('debug', logger.debug);
  forwardConsole('info', logger.info);
  forwardConsole('warn', logger.warn);
  forwardConsole('error', logger.error);

  window.addEventListener('unhandledrejection', (event) => {
    console.error('Unhandled Rejection', event.reason);
  });

  window.addEventListener('error', (event) => {
    // could be undefined on fetch errors
    if (event.error || event.message) {
      console.error('Uncaught Error', event.error || event.message);
    }
  }, true);
}
