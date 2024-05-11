import { wrapConsole } from '../utils/ConsoleWrapper';
import { invoke } from '@tauri-apps/api/core';
import { emitTo } from '@tauri-apps/api/event';
import { getCurrent } from '@tauri-apps/api/webviewWindow';

import './index.css';

async function Main() {
  wrapConsole();
  let view = getCurrent();

  view.listen('init', async () => {
    await getCurrent().show();

    async function onClick(e: MouseEvent | TouchEvent) {
      invoke('ensure_hitboxes_zorder');

      let x = 0;
      let y = 0;
      if (e instanceof MouseEvent) {
        x = e.clientX;
        y = e.clientY;
        emitTo('fancy-toolbar', 'click', { x, y });
        return;
      }

      if (e.touches && e.touches.length > 0) {
        x = e.touches[0]?.clientX || 0;
        y = e.touches[0]?.clientY || 0;
        emitTo('fancy-toolbar', 'click', { x, y });
      }
    }

    document.body.addEventListener('mousemove', () => {
      emitTo('fancy-toolbar', 'mouseenter');
    });
    document.body.addEventListener('click', onClick);
    document.body.addEventListener('touchend', onClick);
  });
}

Main();
