import { createSelector, createSlice, PayloadAction } from '@reduxjs/toolkit';
import { FancyToolbarSettings, UIColors } from 'seelen-core';
import { Placeholder, ToolbarModule } from 'seelen-core';

import { RootState } from './domain';

import { StateBuilder } from '../../../../shared/StateBuilder';

const initialState: RootState = {
  version: 0,
  dateFormat: '',
  isOverlaped: false,
  focused: null,
  placeholder: null,
  settings: new FancyToolbarSettings(),
  env: {},
  // default values of https://learn.microsoft.com/en-us/windows/win32/api/winbase/ns-winbase-system_power_status
  powerStatus: {
    acLineStatus: 255,
    batteryFlag: 255,
    batteryLifePercent: 255,
    systemStatusFlag: 0,
    batteryLifeTime: -1,
    batteryFullLifeTime: -1,
  },
  batteries: [],
  workspaces: [],
  activeWorkspace: null,
  systemTray: [],
  networkAdapters: [],
  networkLocalIp: null,
  online: false,
  wlanBssEntries: [],
  mediaSessions: [],
  mediaOutputs: [],
  mediaInputs: [],
  notifications: [],
  colors: UIColors.default(),
};

export const RootSlice = createSlice({
  name: 'root',
  initialState,
  reducers: {
    ...StateBuilder.reducersFor(initialState),
    setPlaceholder(state, action: PayloadAction<Placeholder | null>) {
      state.placeholder = action.payload;
      state.version++;
    },
    setItemsOnLeft(state, action: PayloadAction<ToolbarModule[]>) {
      if (state.placeholder) {
        state.placeholder.left = action.payload;
      }
    },
    setItemsOnCenter(state, action: PayloadAction<ToolbarModule[]>) {
      if (state.placeholder) {
        state.placeholder.center = action.payload;
      }
    },
    setItemsOnRight(state, action: PayloadAction<ToolbarModule[]>) {
      if (state.placeholder) {
        state.placeholder.right = action.payload;
      }
    },
    removeItem(state, action: PayloadAction<string>) {
      let id = action.payload;
      if (state.placeholder) {
        state.placeholder.left = state.placeholder.left.filter((d) => d.id !== id);
        state.placeholder.center = state.placeholder.center.filter((d) => d.id !== id);
        state.placeholder.right = state.placeholder.right.filter((d) => d.id !== id);
      }
    },
  },
});

export const RootActions = RootSlice.actions;
export const Selectors = StateBuilder.compositeSelector(initialState);

export const selectDefaultOutput = createSelector(Selectors.mediaOutputs, (mediaOutputs) =>
  mediaOutputs.find((d) => d.is_default_multimedia),
);
