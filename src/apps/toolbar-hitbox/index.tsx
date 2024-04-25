import { toPhysicalPixels } from '../utils';
import { wrapConsole } from '../utils/ConsoleWrapper';
import { PhysicalSize } from '@tauri-apps/api/dpi';
import { emitTo } from '@tauri-apps/api/event';
import { getCurrent } from '@tauri-apps/api/webviewWindow';

import './index.css';

async function Main() {
  wrapConsole();
  let view = getCurrent();

  view.listen('init', () => {
    getCurrent().show();
    document.body.addEventListener('mousemove', () => {
      emitTo('fancy-toolbar', 'mouseenter');
    });
  });

  view.listen('resize', (event) => {
    const { height } = event.payload as any;
    getCurrent().setSize(new PhysicalSize(toPhysicalPixels(window.screen.width), height));
  });
}

Main();
