import { Settings } from '@seelen-ui/lib';
import { path } from '@tauri-apps/api';
import { writeTextFile } from '@tauri-apps/plugin-fs';
import yaml from 'js-yaml';
import { cloneDeep, debounce } from 'lodash';

import { store } from '../shared/store/infra';

export const SavePlaceholderAsCustom = debounce(async () => {
  const { placeholder, env } = store.getState();

  if (!placeholder) return;
  const wasCustom = placeholder.info.filename === 'custom.yml';

  const toBeSaved = cloneDeep(placeholder);
  toBeSaved.info.author = env.USERNAME || 'Me';
  toBeSaved.info.displayName = 'Custom';
  toBeSaved.info.description = 'Customized by me';
  toBeSaved.info.filename = 'custom.yml';

  const filePath = await path.join(
    await path.appDataDir(),
    'placeholders',
    toBeSaved.info.filename,
  );

  await writeTextFile(filePath, yaml.dump(toBeSaved));

  if (!wasCustom) {
    const settings = await Settings.getAsync();
    settings.inner.fancyToolbar.placeholder = toBeSaved.info.filename;
    await settings.save();
  }
}, 1000);
