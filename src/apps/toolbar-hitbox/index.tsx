import { wrapConsole } from '../utils/ConsoleWrapper';
import { emitTo } from '@tauri-apps/api/event';
import { getCurrent } from '@tauri-apps/api/webviewWindow';

import './index.css';

async function Main() {
  wrapConsole();
  let view = getCurrent();

  view.listen('init', async () => {
    await getCurrent().show();
    document.body.addEventListener('mousemove', () => {
      emitTo('fancy-toolbar', 'mouseenter');
    });
  });
}

Main();
