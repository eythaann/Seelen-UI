import { defaultTheme } from '../../../../../shared.interfaces';
import { StateBuilder } from '../../../../utils/StateBuilder';
import { savePinnedItems } from './storeApi';
import { createSlice, PayloadAction } from '@reduxjs/toolkit';

import { SeelenWegSlice } from '../../bar/app';

import {
  App,
  AppFromBackground,
  HWND,
  PinnedApp,
  PinnedAppSide,
  RootState,
  SpecialItemType,
  TemporalPinnedApp,
} from './domain';

const initialState: RootState = {
  pinnedOnLeft: [],
  pinnedOnCenter: [],
  pinnedOnRight: [],
  openApps: {},
  theme: defaultTheme,
  settings: SeelenWegSlice.getInitialState(),
};

function removeAppFromState(state: RootState, searched: App) {
  const search = (app: PinnedApp) => app.exe === searched.exe;

  let index = state.pinnedOnLeft.findIndex(search);
  if (index !== -1) {
    state.pinnedOnLeft.splice(index, 1);
    return;
  }

  index = state.pinnedOnCenter.findIndex(search);
  if (index !== -1) {
    state.pinnedOnCenter.splice(index, 1);
    return;
  }

  index = state.pinnedOnRight.findIndex(search);
  if (index !== -1) {
    state.pinnedOnRight.splice(index, 1);
    return;
  }
}

function removeHwnd(state: PinnedApp[], searched: number) {
  for (let i = 0; i < state.length; i++) {
    const current = state[i]!;
    const index = current.opens.findIndex((hwnd) => hwnd === searched);

    if (index !== -1) {
      current.opens.splice(index, 1);
      if (current.type === SpecialItemType.TemporalPin && current.opens.length === 0) {
        state.splice(i, 1);
      }
      break;
    }
  }
}

function findApp(state: RootState, searched: App): PinnedApp | undefined {
  return (
    state.pinnedOnLeft.find((app) => app.exe === searched.exe) ||
    state.pinnedOnCenter.find((app) => app.exe === searched.exe) ||
    state.pinnedOnRight.find((app) => app.exe === searched.exe)
  );
}

export const RootSlice = createSlice({
  name: 'root',
  initialState,
  reducers: {
    unPin(state, action: PayloadAction<App>) {
      const found = findApp(state, action.payload);
      if (found) {
        found.type = SpecialItemType.TemporalPin;
      }
      savePinnedItems(state);
    },
    pinApp(state, action: PayloadAction<{ app: TemporalPinnedApp; side: PinnedAppSide }>) {
      const { app, side } = action.payload;

      const appToPin = findApp(state, app) as PinnedApp;
      if (appToPin) {
        appToPin.type = SpecialItemType.PinnedApp;
      }

      removeAppFromState(state, appToPin);

      switch (side) {
        case PinnedAppSide.LEFT:
          state.pinnedOnLeft.push(appToPin);
          break;
        case PinnedAppSide.CENTER:
          state.pinnedOnCenter.unshift(appToPin);
          break;
        case PinnedAppSide.RIGHT:
          state.pinnedOnRight.push(appToPin);
          break;
        default:
      }
      savePinnedItems(state);
    },
    addOpenApp(state, action: PayloadAction<AppFromBackground>) {
      const app = action.payload;

      state.openApps[app.hwnd] = app;

      const appOnLeft = state.pinnedOnLeft.find((current) => current.exe === app.exe);
      if (appOnLeft) {
        appOnLeft.opens.push(app.hwnd);
        return;
      }

      const appOnCenter = state.pinnedOnCenter.find((current) => current.exe === app.exe);
      if (appOnCenter) {
        appOnCenter.opens.push(app.hwnd);
        return;
      }

      const appOnRight = state.pinnedOnRight.find((current) => current.exe === app.exe);
      if (appOnRight) {
        appOnRight.opens.push(app.hwnd);
        return;
      }

      state.pinnedOnCenter.push({
        type: SpecialItemType.TemporalPin,
        icon: app.icon,
        exe: app.exe,
        execution_path: app.execution_path,
        title: app.exe.split('\\').at(-1) || 'Unknown',
        opens: [app.hwnd],
      });
    },
    updateOpenAppInfo(state, action: PayloadAction<AppFromBackground>) {
      const found = state.openApps[action.payload.hwnd];
      if (found) {
        found.title = action.payload.title;
      }
    },
    removeOpenApp(state, action: PayloadAction<HWND>) {
      delete state.openApps[action.payload];
      removeHwnd(state.pinnedOnLeft, action.payload);
      removeHwnd(state.pinnedOnCenter, action.payload);
      removeHwnd(state.pinnedOnRight, action.payload);
    },
    ...StateBuilder.reducersFor(initialState),
  },
});

export const RootActions = RootSlice.actions;
export const Selectors = StateBuilder.compositeSelector(initialState);
export const SelectOpenApp = (hwnd: HWND) => (state: RootState) => state.openApps[hwnd];

export const isRealPinned = (item: App): boolean => {
  return item.type === SpecialItemType.PinnedApp;
};

export const isTemporalPinned = (item: App): item is TemporalPinnedApp => {
  return item.type === SpecialItemType.TemporalPin;
};
