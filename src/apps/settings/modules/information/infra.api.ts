import { path } from '@tauri-apps/api';
import * as dialog from '@tauri-apps/plugin-dialog';

import { LoadSettingsToStore } from '../shared/infrastructure/store';

export async function LoadCustomConfigFile() {
  const file = await dialog.open({
    defaultPath: await path.homeDir(),
    multiple: false,
    title: 'Select settings file',
    filters: [{ name: 'settings', extensions: ['json'] }],
  });

  if (!file) {
    return;
  }

  LoadSettingsToStore(file.path);
}