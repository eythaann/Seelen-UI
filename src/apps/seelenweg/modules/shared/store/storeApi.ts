import { SwSavedItem, SwSaveFile } from '../../../../shared/schemas/SeelenWegItems';
import { path } from '@tauri-apps/api';
import { exists, readTextFile, writeTextFile } from '@tauri-apps/plugin-fs';
import yaml from 'js-yaml';

import { isRealPinned, isTemporalPinned } from './app';

import { RootState, SwItem } from './domain';

export const savePinnedItems = async (state: RootState): Promise<void> => {
  const cb = (acc: SwSavedItem[], app: SwItem) => {
    if (isTemporalPinned(app)) {
      return acc;
    }

    if (isRealPinned(app)) {
      acc.push({
        exe: app.exe,
        type: app.type,
        execution_path: app.execution_path,
        icon_path: app.icon_path,
      });
    }

    return acc;
  };

  const data: SwSaveFile = {
    left: state.itemsOnLeft.reduce(cb, []),
    center: state.itemsOnCenter.reduce(cb, []),
    right: state.itemsOnRight.reduce(cb, []),
  };

  const yaml_route = await path.join(await path.homeDir(), '.config/seelen/seelenweg_items.yaml');
  await writeTextFile(yaml_route, yaml.dump(data));
};

export const loadPinnedItems = async (): Promise<SwSaveFile> => {
  let yaml_route = await path.join(await path.homeDir(), '.config/seelen/seelenweg_items.yaml');

  if (!(await exists(yaml_route))) {
    return {
      left: [],
      center: [],
      right: [],
    };
  }

  const yaml_data: any = yaml.load(await readTextFile(yaml_route));
  return {
    left: yaml_data?.left || [],
    center: yaml_data?.center || [],
    right: yaml_data?.right || [],
  };
};
