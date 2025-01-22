import { path } from '@tauri-apps/api';
import { writeTextFile } from '@tauri-apps/plugin-fs';
import yaml from 'js-yaml';
import { cloneDeep, debounce } from 'lodash';

import { store } from '../shared/store/infra';

export const SavePlaceholderAsCustom = debounce(async () => {
  const { placeholder } = store.getState();
  const toBeSaved = cloneDeep(placeholder);
  const filePath = await path.join(await path.appDataDir(), 'toolbar_items.yml');
  await writeTextFile(filePath, yaml.dump(toBeSaved));
}, 1000);
