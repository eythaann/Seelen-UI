import { WegItems, WegItemType } from '@seelen-ui/lib';
import { debounce } from 'lodash';

import { store } from './infra';

import { RootState, SwItem } from './domain';

export const IsSavingPinnedItems = {
  current: false,
};

export const savePinnedItems = debounce(
  async (state: RootState = store.getState()): Promise<void> => {
    const cb = (item: SwItem) => {
      return item.type !== WegItemType.Temporal;
    };

    const data = new WegItems({
      isReorderDisabled: false,
      left: state.itemsOnLeft.filter(cb),
      center: state.itemsOnCenter.filter(cb),
      right: state.itemsOnRight.filter(cb),
    });

    await data.save();
  },
  1000,
);
