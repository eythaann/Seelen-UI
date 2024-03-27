import { Theme } from '../../../../../shared.interfaces';
import { StateBuilder } from '../../../../utils/StateBuilder';
import { createSlice, PayloadAction } from '@reduxjs/toolkit';

import { SeelenWegSlice } from '../../bar/app';

import { App, OpenApp, PinnedApp, PinnedAppSide, RootState, SpecialItemType } from './domain';

const defaultTheme: Theme = {
  info: {
    filename: 'unknown',
    displayName: 'Empty',
    author: 'none',
  },
  seelenweg: {
    background: [],
    items: {
      background: [],
    },
    contextMenu: {
      background: [],
    },
  },
};

const initialState: RootState = {
  apps: [],
  pinnedOnLeft: [],
  pinnedOnCenter: [],
  pinnedOnRight: [],
  theme: defaultTheme,
  settings: SeelenWegSlice.getInitialState(),
};

export const RootSlice = createSlice({
  name: 'root',
  initialState,
  reducers: {
    unPin(state, action: PayloadAction<App>) {
      const search = (app: PinnedApp) => app.exe === action.payload.exe;

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
    },
    pinApp(state, action: PayloadAction<{ app: OpenApp; side: PinnedAppSide }>) {
      const { app, side } = action.payload;
      const appToPin: PinnedApp = {
        type: SpecialItemType.App,
        icon: app.icon,
        exe: app.exe,
        title: app.exe.split('\\').at(-1) || 'Unknown',
      };
      switch (side) {
        case PinnedAppSide.LEFT:
          state.pinnedOnLeft.push(appToPin);
          break;
        case PinnedAppSide.CENTER:
          state.pinnedOnCenter.push(appToPin);
          break;
        case PinnedAppSide.RIGHT:
          state.pinnedOnRight.push(appToPin);
          break;
        default:
      }
    },
    addOpenApp(state, action: PayloadAction<OpenApp>) {
      state.apps.push(action.payload);
    },
    removeOpenApp(state, action: PayloadAction<number>) {
      let index = state.apps.findIndex((app) => app.hwnd === action.payload);
      if (index !== -1) {
        state.apps.splice(index, 1);
      }
    },
    ...StateBuilder.reducersFor(initialState),
  },
});

export const RootActions = RootSlice.actions;
export const Selectors = StateBuilder.compositeSelector(initialState);

export const isAppPinned = (state: RootState, item: App) => {
  const search = (app: App) => app.exe === item.exe;
  return (
    !!state.pinnedOnLeft.find(search) ||
    !!state.pinnedOnCenter.find(search) ||
    !!state.pinnedOnRight.find(search)
  );
};
