import { Placeholder, ToolbarModule } from '../../../../shared/schemas/Placeholders';
import { StateBuilder } from '../../../../shared/StateBuilder';
import { createSelector, createSlice, PayloadAction } from '@reduxjs/toolkit';
import { FancyToolbarSettings, UIColors } from 'seelen-core';

import { RootState } from './domain';

const initialState: RootState = {
  version: 0,
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
  },
});

export const RootActions = RootSlice.actions;
export const Selectors = StateBuilder.compositeSelector(initialState);

export const selectDefaultOutput = createSelector(Selectors.mediaOutputs, (mediaOutputs) =>
  mediaOutputs.find((d) => d.is_default_multimedia),
);
