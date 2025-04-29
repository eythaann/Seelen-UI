import { Placeholder, ToolbarItem } from '@seelen-ui/lib/types';
import { path } from '@tauri-apps/api';
import { writeTextFile } from '@tauri-apps/plugin-fs';
import yaml from 'js-yaml';
import { cloneDeep, debounce, throttle } from 'lodash';

import { store } from '../shared/store/infra';

export const SaveToolbarItems = debounce(async () => {
  const { items: placeholder } = store.getState();
  const toBeSaved = cloneDeep(placeholder);
  const filePath = await path.join(await path.appDataDir(), 'toolbar_items.yml');
  await writeTextFile(filePath, yaml.dump(toBeSaved));
}, 1000);

export const RestoreToDefault = throttle(async () => {
  const { items: placeholder } = store.getState();

  // based on src\background\state\application\toolbar_items.rs
  const toBeSaved: Placeholder = {
    ...placeholder,
    left: [
      '@default/user-folder',
      { id: crypto.randomUUID(), type: 'text', template: 'return "|"' } as ToolbarItem,
      '@default/focused-app',
      {
        id: crypto.randomUUID(),
        type: 'generic',
        template: 'return window.title ? "-" : ""',
      } as ToolbarItem,
      '@default/focused-app-title',
    ],
    center: [
      '@default/date',
    ],
    right: [
      '@default/system-tray',
      '@default/keyboard',
      '@default/bluetooth',
      '@default/network',
      '@default/media',
      '@default/power',
      '@default/notifications',
      '@default/quick-settings',
    ],
  };

  const filePath = await path.join(await path.appDataDir(), 'toolbar_items.yml');
  await writeTextFile(filePath, yaml.dump(toBeSaved));
}, 2000);
