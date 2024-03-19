import { getCurrent } from '@tauri-apps/api/webviewWindow';
import { createRoot } from 'react-dom/client';

import { loadUserSettings } from '../settings/modules/shared/infrastructure/storeApi';

import { App } from './app';

import './styles/colors.css';
import './styles/reset.css';
import './styles/global.css';

async function main() {
  const userSettings = await loadUserSettings();

  if (!userSettings.updateNotification) {
    getCurrent().close();
    return;
  }

  const root = createRoot(document.getElementById('root') || document.body);
  root.render(<App/>);
}

main();