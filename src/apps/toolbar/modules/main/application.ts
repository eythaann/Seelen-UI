import {
  saveJsonSettings,
  UserSettingsLoader,
} from '../../../settings/modules/shared/store/storeApi';
import { path } from '@tauri-apps/api';
import { writeTextFile } from '@tauri-apps/plugin-fs';
import yaml from 'js-yaml';
import { cloneDeep } from 'lodash';

import { store } from '../shared/store/infra';

export async function SavePlaceholderAsCustom() {
  const { placeholder, env } = store.getState();

  if (!placeholder) return;

  const toBeSaved = cloneDeep(placeholder);
  toBeSaved.info.author = env.USERNAME || 'Me';
  toBeSaved.info.displayName = 'Custom';
  toBeSaved.info.description = 'Customized by me';
  toBeSaved.info.filename = 'custom.yml';

  const filePath = await path.join(await path.appDataDir(), 'placeholders', toBeSaved.info.filename);

  await writeTextFile(filePath, yaml.dump(toBeSaved));

  let { jsonSettings } = await new UserSettingsLoader().withThemes(false).load();
  jsonSettings.fancyToolbar.placeholder = toBeSaved.info.filename;

  saveJsonSettings(jsonSettings);
}
