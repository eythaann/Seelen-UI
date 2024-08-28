import { wrapConsole } from '../shared/ConsoleWrapper';
import { invoke } from '@tauri-apps/api/core';
import { emitTo } from '@tauri-apps/api/event';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';

import './index.css';

async function Main() {
  wrapConsole();
  let view = getCurrentWebviewWindow();
  let main = view.label.replace('-hitbox', '');

  view.listen('init', async () => {
    await getCurrentWebviewWindow().show();

    async function onClick(e: MouseEvent | TouchEvent) {
      invoke('ensure_hitboxes_zorder').catch(console.error);

      let x = 0;
      let y = 0;
      if (e instanceof MouseEvent) {
        x = e.clientX;
        y = e.clientY;
        emitTo(main, 'click', { x, y });
        return;
      }

      if (e.touches && e.touches.length > 0) {
        x = e.touches[0]?.clientX || 0;
        y = e.touches[0]?.clientY || 0;
        emitTo(main, 'click', { x, y });
      }
    }

    document.body.addEventListener('mouseenter', () => {
      emitTo(main, 'mouseenter');
      invoke('ensure_hitboxes_zorder').catch(console.error);
    });
    document.body.addEventListener('click', onClick);
    document.body.addEventListener('touchend', onClick);
  });

  view.listen('debug-hitbox', () => {
    document.body.classList.toggle('debug-hitbox');
  });
}

Main();
