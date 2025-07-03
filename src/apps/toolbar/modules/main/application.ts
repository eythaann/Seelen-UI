import { fs } from '@seelen-ui/lib/tauri';
import { Placeholder, PluginId, ToolbarItem } from '@seelen-ui/lib/types';
import { path } from '@tauri-apps/api';
import yaml from 'js-yaml';
import { cloneDeep, debounce, throttle } from 'lodash';

import { store } from '../shared/store/infra';

export const SaveToolbarItems = debounce(async () => {
  const { items: placeholder } = store.getState();
  const toBeSaved = cloneDeep(placeholder);
  const filePath = await path.join(await path.appDataDir(), 'toolbar_items.yml');
  await fs.writeTextFile(filePath, yaml.dump(toBeSaved));
}, 1000);

export const RestoreToDefault = throttle(async () => {
  const { items: placeholder } = store.getState();

  // based on src\background\state\application\toolbar_items.rs
  const toBeSaved: Placeholder = {
    ...placeholder,
    left: [
      '@default/user-folder' as PluginId,
      { id: crypto.randomUUID(), type: 'text', template: 'return "|"' } as ToolbarItem,
      '@default/focused-app' as PluginId,
      {
        id: crypto.randomUUID(),
        type: 'generic',
        template: 'return window.title ? "-" : ""',
      } as ToolbarItem,
      '@default/focused-app-title' as PluginId,
    ],
    center: ['@default/date' as PluginId],
    right: [
      '@default/system-tray' as PluginId,
      '@default/keyboard' as PluginId,
      '@default/bluetooth' as PluginId,
      '@default/network' as PluginId,
      '@default/media' as PluginId,
      '@default/power' as PluginId,
      '@default/notifications' as PluginId,
      '@default/quick-settings' as PluginId,
    ],
  };

  const filePath = await path.join(await path.appDataDir(), 'toolbar_items.yml');
  await fs.writeTextFile(filePath, yaml.dump(toBeSaved));
}, 2000);
