import { defaultTheme } from '../../../../../shared.interfaces';
import { StateBuilder } from '../../../../shared/StateBuilder';
import { savePinnedItems } from './storeApi';
import { createSlice, PayloadAction } from '@reduxjs/toolkit';

import { SeelenWegSlice } from '../../bar/app';
import { PinnedApp } from '../../item/app/PinnedApp';
import { TemporalApp } from '../../item/app/TemporalApp';

import { App, AppFromBackground, AppsSides, HWND, RootState, SpecialItemType } from './domain';

const initialState: RootState = {
  pinnedOnLeft: [],
  pinnedOnCenter: [],
  pinnedOnRight: [],
  openApps: {},
  focusedHandle: 0,
  themeLayers: defaultTheme.layers,
  isOverlaped: false,
  settings: SeelenWegSlice.getInitialState(),
};

function removeAppFromState(state: RootState, searched: App) {
  const search = (app: App) => app.exe === searched.exe;

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

function removeHwnd(state: App[], searched: HWND) {
  for (let i = 0; i < state.length; i++) {
    const current = state[i]!;
    if (
      current.type !== SpecialItemType.PinnedApp &&
      current.type !== SpecialItemType.TemporalPin
    ) {
      continue;
    }

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

function findApp(state: RootState, searched: App): App | undefined {
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
    ...StateBuilder.reducersFor(initialState),
    unPin(state, action: PayloadAction<App>) {
      const found = findApp(state, action.payload);
      if (found) {
        found.type = SpecialItemType.TemporalPin;
        if (found.opens.length === 0) {
          removeAppFromState(state, found);
        }
      }
    },
    pinApp(state, action: PayloadAction<{ app: TemporalApp; side: AppsSides }>) {
      const { app, side } = action.payload;

      const appToPin = findApp(state, app) as PinnedApp;
      if (appToPin) {
        appToPin.type = SpecialItemType.PinnedApp;
      }

      removeAppFromState(state, appToPin);

      switch (side) {
        case AppsSides.LEFT:
          state.pinnedOnLeft.unshift(appToPin);
          break;
        case AppsSides.CENTER:
          state.pinnedOnCenter.unshift(appToPin);
          break;
        case AppsSides.RIGHT:
          state.pinnedOnRight.push(appToPin);
          break;
        default:
      }
    },
    addOpenApp(state, action: PayloadAction<AppFromBackground>) {
      const app = action.payload;

      if (state.openApps[app.hwnd]) {
        return;
      }

      state.openApps[app.hwnd] = app;

      const appFilename = app.exe.split('\\').pop();
      if (appFilename) {
        const pinedApp =
          state.pinnedOnLeft.find((current) => current.exe.endsWith(appFilename)) ||
          state.pinnedOnCenter.find((current) => current.exe.endsWith(appFilename)) ||
          state.pinnedOnRight.find((current) => current.exe.endsWith(appFilename));

        if (pinedApp) {
          pinedApp.opens.push(app.hwnd);

          if (pinedApp.exe !== app.exe) {
            pinedApp.exe = app.exe;
            pinedApp.execution_path = app.execution_path;
            savePinnedItems(state);
          }
          return;
        }
      }

      state.pinnedOnCenter.push(TemporalApp.fromBackground(app));
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
  },
});

export const RootActions = RootSlice.actions;
export const Selectors = StateBuilder.compositeSelector(initialState);
export const SelectOpenApp = (hwnd: HWND) => (state: RootState) => state.openApps[hwnd];

export const isRealPinned = (item: App): item is PinnedApp => {
  return item.type === SpecialItemType.PinnedApp;
};

export const isTemporalPinned = (item: App): item is TemporalApp => {
  return item.type === SpecialItemType.TemporalPin;
};
