import { createSlice, current, PayloadAction } from '@reduxjs/toolkit';
import { Settings, UIColors, WegItemType } from '@seelen-ui/lib';

import {
  AppsSides,
  HWND,
  PinnedWegItem,
  RootState,
  SwItem,
  TemporalWegItem,
} from './domain';

import { StateBuilder } from '../../../../shared/StateBuilder';
import { savePinnedItems } from './storeApi';

const initialState: RootState = {
  devTools: false,
  itemsOnLeft: [],
  itemsOnCenter: [],
  itemsOnRight: [],
  openApps: {},
  focusedApp: null,
  isOverlaped: false,
  settings: (await Settings.default()).inner.seelenweg,
  mediaSessions: [],
  colors: UIColors.default().inner,
};

function removeAppFromState(
  state: RootState,
  searched: PinnedWegItem | TemporalWegItem,
) {
  const search = (app: SwItem) => 'execution_command' in app && app.execution_command === searched.execution_command;

  let index = state.itemsOnLeft.findIndex(search);
  if (index !== -1) {
    state.itemsOnLeft.splice(index, 1);
    return;
  }

  index = state.itemsOnCenter.findIndex(search);
  if (index !== -1) {
    state.itemsOnCenter.splice(index, 1);
    return;
  }

  index = state.itemsOnRight.findIndex(search);
  if (index !== -1) {
    state.itemsOnRight.splice(index, 1);
    return;
  }
}

function findApp(
  state: RootState,
  searched: PinnedWegItem | TemporalWegItem,
) {
  return (state.itemsOnLeft.find(
    (app) => 'execution_command' in app && app.execution_command === searched.execution_command,
  ) ||
    state.itemsOnCenter.find(
      (app) => 'execution_command' in app && app.execution_command === searched.execution_command,
    ) ||
    state.itemsOnRight.find(
      (app) => 'execution_command' in app && app.execution_command === searched.execution_command,
    )) as PinnedWegItem | TemporalWegItem | undefined;
}

export const RootSlice = createSlice({
  name: 'root',
  initialState,
  reducers: {
    ...StateBuilder.reducersFor(initialState),
    unpin(state, action: PayloadAction<PinnedWegItem>) {
      const filter = (item: any) => !('path' in item) || item.path !== action.payload.path;
      state.itemsOnLeft = state.itemsOnLeft.filter(filter);
      state.itemsOnCenter = state.itemsOnCenter.filter(filter);
      state.itemsOnRight = state.itemsOnRight.filter(filter);
    },
    pinApp(state, action: PayloadAction<{ app: TemporalWegItem; side: AppsSides }>) {
      const { app, side } = action.payload;

      const appToPin = findApp(state, app) || app;
      appToPin.type = WegItemType.Pinned;

      switch (side) {
        case AppsSides.Left:
          removeAppFromState(state, appToPin);
          state.itemsOnLeft.unshift(appToPin);
          break;
        case AppsSides.Center:
          removeAppFromState(state, appToPin);
          state.itemsOnCenter.unshift(appToPin);
          break;
        case AppsSides.Right:
          removeAppFromState(state, appToPin);
          state.itemsOnRight.push(appToPin);
          break;
        default:
      }
      savePinnedItems(current(state));
    },
    unPinApp(state, action: PayloadAction<PinnedWegItem | TemporalWegItem>) {
      const found = findApp(state, action.payload);
      if (found) {
        found.type = WegItemType.Temporal;
        if (found.windows.length === 0) {
          removeAppFromState(state, found);
        }
        savePinnedItems(current(state));
      }
    },
    addMediaModule(state) {
      const all = [...state.itemsOnLeft, ...state.itemsOnCenter, ...state.itemsOnRight];
      if (!all.some((current) => current.type === WegItemType.Media)) {
        state.itemsOnRight.push({
          type: WegItemType.Media,
        });
      }
      savePinnedItems(current(state));
    },
    removeMediaModule(state) {
      const filter = (current: SwItem) => current.type !== WegItemType.Media;
      state.itemsOnLeft = state.itemsOnLeft.filter(filter);
      state.itemsOnCenter = state.itemsOnCenter.filter(filter);
      state.itemsOnRight = state.itemsOnRight.filter(filter);
      savePinnedItems(current(state));
    },
    addStartModule(state) {
      const all = [...state.itemsOnLeft, ...state.itemsOnCenter, ...state.itemsOnRight];
      if (!all.some((current) => current.type === WegItemType.StartMenu)) {
        state.itemsOnLeft.unshift({
          type: WegItemType.StartMenu,
        });
      }
      savePinnedItems(current(state));
    },
    removeStartModule(state) {
      const filter = (current: SwItem) => current.type !== WegItemType.StartMenu;
      state.itemsOnLeft = state.itemsOnLeft.filter(filter);
      state.itemsOnCenter = state.itemsOnCenter.filter(filter);
      state.itemsOnRight = state.itemsOnRight.filter(filter);
      savePinnedItems(current(state));
    },
  },
});

export const RootActions = RootSlice.actions;
export const Selectors = StateBuilder.compositeSelector(initialState);
export const SelectOpenApp = (hwnd: HWND) => (state: RootState) => state.openApps[hwnd];

export const isPinnedApp = (item: SwItem): item is PinnedWegItem => {
  return item.type === WegItemType.Pinned;
};

export const isTemporalApp = (item: SwItem): item is TemporalWegItem => {
  return item.type === WegItemType.Temporal;
};
