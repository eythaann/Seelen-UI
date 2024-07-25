import {
  SwItemType,
  SwSavedItem,
  SwSaveFile,
  SwSaveFileSchema,
} from '../../../../shared/schemas/SeelenWegItems';
import { path } from '@tauri-apps/api';
import { exists, readTextFile, writeTextFile } from '@tauri-apps/plugin-fs';
import yaml from 'js-yaml';
import { debounce } from 'lodash';

import { store } from './infra';

import { RootState, SwItem } from './domain';

export const savePinnedItems = debounce(
  async (state: RootState = store.getState()): Promise<void> => {
    const cb = (acc: SwSavedItem[], item: SwItem) => {
      switch (item.type) {
        case SwItemType.TemporalApp:
          break;
        case SwItemType.PinnedApp:
          acc.push({
            type: item.type,
            exe: item.exe,
            execution_path: item.execution_path,
            icon_path: item.icon_path,
          });
          break;
        default:
          acc.push(item);
          break;
      }
      return acc;
    };

    const data: SwSaveFile = {
      left: state.itemsOnLeft.reduce(cb, []),
      center: state.itemsOnCenter.reduce(cb, []),
      right: state.itemsOnRight.reduce(cb, []),
    };

    const yaml_route = await path.join(await path.appDataDir(), 'seelenweg_items.yaml');
    await writeTextFile(yaml_route, yaml.dump(data));
  },
  1000,
);

export const loadPinnedItems = async (): Promise<SwSaveFile> => {
  let yaml_route = await path.join(await path.appDataDir(), 'seelenweg_items.yaml');

  if (!(await exists(yaml_route))) {
    return SwSaveFileSchema.parse({});
  }

  return SwSaveFileSchema.parse(yaml.load(await readTextFile(yaml_route)));
};
