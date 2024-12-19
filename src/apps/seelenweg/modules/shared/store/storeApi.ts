import { WegItemType } from '@seelen-ui/lib';
import { WegItem, WegItems } from '@seelen-ui/lib/types';
import { path } from '@tauri-apps/api';
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
    const cb = (acc: WegItem[], item: SwItem) => {
      switch (item.type) {
        case WegItemType.Temporal:
          break;
        case WegItemType.Pinned:
          acc.push({
            type: item.type,
            path: item.path,
            execution_command: item.execution_command,
            is_dir: item.is_dir,
          });
          break;
        default:
          acc.push(item);
          break;
      }
      return acc;
    };

    const data: WegItems = {
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
