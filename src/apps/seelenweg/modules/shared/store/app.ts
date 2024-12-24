import { createSlice, current, PayloadAction } from '@reduxjs/toolkit';
import { Settings, UIColors, WegItemType } from '@seelen-ui/lib';
import { WegItem } from '@seelen-ui/lib/types';

import { PinnedWegItem, RootState, SwItem, TemporalWegItem } from './domain';

import { StateBuilder } from '../../../../shared/StateBuilder';
import { savePinnedItems } from './storeApi';

const initialState: RootState = {
  devTools: false,
  itemsOnLeft: [],
  itemsOnCenter: [],
  itemsOnRight: [],
  focusedApp: null,
  isOverlaped: false,
  settings: (await Settings.default()).inner.seelenweg,
  mediaSessions: [],
  colors: UIColors.default().inner,
};

function findApp(state: RootState, id: string): WegItem | null {
  const cb = (app: WegItem): app is PinnedWegItem => app.id === id;
  return (
    state.itemsOnLeft.find(cb) ||
    state.itemsOnCenter.find(cb) ||
    state.itemsOnRight.find(cb) ||
    null
  );
}

export const RootSlice = createSlice({
  name: 'root',
  initialState,
  reducers: {
    ...StateBuilder.reducersFor(initialState),
    remove(state, action: PayloadAction<string>) {
      const filter = (item: WegItem) => item.id !== action.payload;
      state.itemsOnLeft = state.itemsOnLeft.filter(filter);
      state.itemsOnCenter = state.itemsOnCenter.filter(filter);
      state.itemsOnRight = state.itemsOnRight.filter(filter);
      savePinnedItems(current(state));
    },
    pinApp(state, action: PayloadAction<string>) {
      const item = findApp(state, action.payload);
      if (item) {
        item.type = WegItemType.Pinned;
        savePinnedItems(current(state));
      }
    },
    unPinApp(state, action: PayloadAction<string>) {
      const item = findApp(state, action.payload);
      if (item) {
        item.type = WegItemType.Temporal;
        savePinnedItems(current(state));
      }
    },
    addMediaModule(state) {
      const all = [...state.itemsOnLeft, ...state.itemsOnCenter, ...state.itemsOnRight];
      if (!all.some((current) => current.type === WegItemType.Media)) {
        state.itemsOnRight.push({
          id: crypto.randomUUID(),
          type: WegItemType.Media,
        });
      }
      savePinnedItems(current(state));
    },
    addStartModule(state) {
      const all = [...state.itemsOnLeft, ...state.itemsOnCenter, ...state.itemsOnRight];
      if (!all.some((current) => current.type === WegItemType.StartMenu)) {
        state.itemsOnLeft.unshift({
          id: crypto.randomUUID(),
          type: WegItemType.StartMenu,
        });
      }
      savePinnedItems(current(state));
    },
  },
});

export const RootActions = RootSlice.actions;
export const Selectors = StateBuilder.compositeSelector(initialState);

export const isPinnedApp = (item: SwItem): item is PinnedWegItem => {
  return item.type === WegItemType.Pinned;
};

export const isTemporalApp = (item: SwItem): item is TemporalWegItem => {
  return item.type === WegItemType.Temporal;
};
