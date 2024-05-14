import { wrapConsole } from '../utils/ConsoleWrapper';
import { invoke } from '@tauri-apps/api/core';
import { PhysicalPosition, PhysicalSize } from '@tauri-apps/api/dpi';
import { emitTo } from '@tauri-apps/api/event';
import { getCurrent } from '@tauri-apps/api/webviewWindow';

import './index.css';

async function Main() {
  wrapConsole();
  let view = getCurrent();
  let main = view.label.replace('-hitbox', '');

  view.listen('init', () => {
    getCurrent().show();

    document.body.addEventListener('mousemove', () => {
      emitTo(main, 'mouseenter');
    });

    async function onClick(e: MouseEvent | TouchEvent) {
      invoke('ensure_hitboxes_zorder');

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

    document.body.addEventListener('click', onClick);
    document.body.addEventListener('touchend', onClick);
  });

  view.listen('resize', (event) => {
    const { width, height } = event.payload as any;
    getCurrent().setSize(new PhysicalSize(width, height));
  });

  view.listen('move', (event) => {
    const { x, y } = event.payload as any;
    getCurrent().setPosition(new PhysicalPosition(x, y));
  });
}

Main();
