// before tauri v2.5 this script was done as a workaround to https://github.com/tauri-apps/tauri/issues/12348
// but was fixed on https://github.com/tauri-apps/wry/pull/1531 so now this script is used to initialize
// the console logger to capture any error on the main script

import { wrapConsoleV2 } from './ConsoleWrapper';

/* async function WaitForTauriInternals(): Promise<void> {
  await new Promise<void>((resolve) => {
    const checkInterval = setInterval(() => {
      if ('__TAURI_INTERNALS__' in window) {
        clearInterval(checkInterval);
        resolve();
      }
    }, 10);
  });
}

await WaitForTauriInternals(); */
wrapConsoleV2();

/* const script = document.createElement('script');
script.src = './index.js';
script.type = 'module';
script.defer = true;
document.head.appendChild(script); */

export {};
