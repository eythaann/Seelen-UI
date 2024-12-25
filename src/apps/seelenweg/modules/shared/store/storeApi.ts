import { WegItems, WegItemType } from '@seelen-ui/lib';
import { WegItem } from '@seelen-ui/lib/types';
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
        default:
          acc.push(item);
          break;
      }
      return acc;
    };

    const data = new WegItems({
      left: state.itemsOnLeft.reduce(cb, []),
      center: state.itemsOnCenter.reduce(cb, []),
      right: state.itemsOnRight.reduce(cb, []),
    });

    await data.save();
  },
  1000,
);
