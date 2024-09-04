import { StateBuilder } from '../../../../shared/StateBuilder';
import { savePinnedItems } from './storeApi';
import { createSlice, current, PayloadAction } from '@reduxjs/toolkit';
import { SeelenWegSettings, UIColors } from 'seelen-core';

import { SwTemporalAppUtils } from '../../item/app/TemporalApp';

import {
  AppFromBackground,
  AppsSides,
  HWND,
  RootState,
  SpecialItemType,
  SwItem,
  SwPinnedApp,
  SwTemporalApp,
} from './domain';

const initialState: RootState = {
  itemsOnLeft: [],
  itemsOnCenter: [],
  itemsOnRight: [],
  openApps: {},
  focusedApp: null,
  isOverlaped: false,
  settings: new SeelenWegSettings(),
  mediaSessions: [],
  colors: UIColors.default(),
};

function removeAppFromState(state: RootState, searched: SwPinnedApp | SwTemporalApp) {
  const search = (app: SwItem) => 'exe' in app && app.exe === searched.exe;

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

function findApp(state: RootState, searched: SwPinnedApp | SwTemporalApp) {
  return (state.itemsOnLeft.find((app) => 'exe' in app && app.exe === searched.exe) ||
    state.itemsOnCenter.find((app) => 'exe' in app && app.exe === searched.exe) ||
    state.itemsOnRight.find((app) => 'exe' in app && app.exe === searched.exe)) as
    | SwPinnedApp
    | SwTemporalApp
    | undefined;
}

export const RootSlice = createSlice({
  name: 'root',
  initialState,
  reducers: {
    ...StateBuilder.reducersFor(initialState),
    unPin(state, action: PayloadAction<SwPinnedApp | SwTemporalApp>) {
      const found = findApp(state, action.payload);
      if (found) {
        found.type = SpecialItemType.TemporalApp;
        if (found.opens.length === 0) {
          removeAppFromState(state, found);
        }
      }
    },
    pinApp(state, action: PayloadAction<{ app: SwTemporalApp; side: AppsSides }>) {
      const { app, side } = action.payload;

      const appToPin = findApp(state, app) || app;
      appToPin.type = SpecialItemType.PinnedApp;

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
    },
    addMediaModule(state) {
      const all = [...state.itemsOnLeft, ...state.itemsOnCenter, ...state.itemsOnRight];
      if (!all.some((current) => current.type === SpecialItemType.Media)) {
        state.itemsOnRight.push({
          type: SpecialItemType.Media,
        });
      }
      savePinnedItems(current(state));
    },
    removeMediaModule(state) {
      const filter = (current: SwItem) => current.type !== SpecialItemType.Media;
      state.itemsOnLeft = state.itemsOnLeft.filter(filter);
      state.itemsOnCenter = state.itemsOnCenter.filter(filter);
      state.itemsOnRight = state.itemsOnRight.filter(filter);
      savePinnedItems(current(state));
    },
    addStartModule(state) {
      const all = [...state.itemsOnLeft, ...state.itemsOnCenter, ...state.itemsOnRight];
      if (!all.some((current) => current.type === SpecialItemType.Start)) {
        state.itemsOnLeft.unshift({
          type: SpecialItemType.Start,
        });
      }
      savePinnedItems(current(state));
    },
    removeStartModule(state) {
      const filter = (current: SwItem) => current.type !== SpecialItemType.Start;
      state.itemsOnLeft = state.itemsOnLeft.filter(filter);
      state.itemsOnCenter = state.itemsOnCenter.filter(filter);
      state.itemsOnRight = state.itemsOnRight.filter(filter);
      savePinnedItems(current(state));
    },
    addOpenApp(state, action: PayloadAction<AppFromBackground>) {
      const app = action.payload;

      state.openApps[app.hwnd] = app;

      const appFilename = app.exe.split('\\').pop();
      if (appFilename) {
        const cb = (current: SwItem) => 'exe' in current && current.exe.endsWith(appFilename);
        const pinedApp = (state.itemsOnLeft.find(cb) ||
          state.itemsOnCenter.find(cb) ||
          state.itemsOnRight.find(cb)) as SwPinnedApp | undefined;

        if (pinedApp) {
          if (!pinedApp.opens.includes(app.hwnd)) {
            pinedApp.opens.push(app.hwnd);
          }

          // update path to pinned apps normally changed on updates
          if (pinedApp.exe !== app.exe) {
            pinedApp.exe = app.exe;
            pinedApp.execution_path = app.execution_path;
            savePinnedItems(current(state));
          }
          return;
        }
      }

      state.itemsOnCenter.push(SwTemporalAppUtils.fromBackground(app));
    },
    updateOpenAppInfo(state, action: PayloadAction<AppFromBackground>) {
      const found = state.openApps[action.payload.hwnd];
      if (found) {
        found.title = action.payload.title;
      }
    },
    removeOpenApp(state, action: PayloadAction<HWND>) {
      delete state.openApps[action.payload];

      function filter(app: SwItem) {
        if ('opens' in app) {
          app.opens = app.opens.filter((hwnd) => hwnd !== action.payload);
        }
        return app.type !== SpecialItemType.TemporalApp || app.opens.length > 0;
      }

      state.itemsOnLeft = state.itemsOnLeft.filter(filter);
      state.itemsOnCenter = state.itemsOnCenter.filter(filter);
      state.itemsOnRight = state.itemsOnRight.filter(filter);
    },
  },
});

export const RootActions = RootSlice.actions;
export const Selectors = StateBuilder.compositeSelector(initialState);
export const SelectOpenApp = (hwnd: HWND) => (state: RootState) => state.openApps[hwnd];

export const isPinnedApp = (item: SwItem): item is SwPinnedApp => {
  return item.type === SpecialItemType.PinnedApp;
};

export const isTemporalApp = (item: SwItem): item is SwTemporalApp => {
  return item.type === SpecialItemType.TemporalApp;
};
