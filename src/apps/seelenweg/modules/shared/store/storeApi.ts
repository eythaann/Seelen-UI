import {
  SwItemType,
  SwSavedItem,
  SwSaveFile,
  SwSaveFileSchema,
} from '../../../../shared/schemas/SeelenWegItems';
import { path } from '@tauri-apps/api';
import { invoke } from '@tauri-apps/api/core';
import { writeTextFile } from '@tauri-apps/plugin-fs';
import yaml from 'js-yaml';
import { debounce } from 'lodash';

import { store } from './infra';

import { RootState, SwItem } from './domain';

export const IsSavingPinnedItems = {
  current: false,
};

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
    IsSavingPinnedItems.current = true;
    await writeTextFile(yaml_route, yaml.dump(data));
  },
  1000,
);

export const loadPinnedItems = async (): Promise<SwSaveFile> => {
  let items = await invoke<json>('state_get_weg_items');
  return SwSaveFileSchema.parse(items || {});
};
