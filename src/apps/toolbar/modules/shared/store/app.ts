import { createSelector, createSlice, PayloadAction } from '@reduxjs/toolkit';
import { Settings, UIColors } from '@seelen-ui/lib';
import { Placeholder, ToolbarItem } from '@seelen-ui/lib/types';

import { RootState } from './domain';

import { StateBuilder } from '../../../../shared/StateBuilder';

const settings = await Settings.default();

const initialState: RootState = {
  version: 0,
  items: {
    left: [],
    center: [],
    right: [],
  },
  plugins: [],
  dateFormat: '',
  isOverlaped: false,
  focused: null,
  settings: settings.fancyToolbar,
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
  colors: UIColors.default().inner,
};

export const RootSlice = createSlice({
  name: 'root',
  initialState,
  reducers: {
    ...StateBuilder.reducersFor(initialState),
    setPlaceholder(state, action: PayloadAction<Placeholder>) {
      state.items = action.payload;
      state.version++;
    },
    setItemsOnLeft(state, action: PayloadAction<ToolbarItem[]>) {
      if (state.items) {
        state.items.left = action.payload;
      }
    },
    setItemsOnCenter(state, action: PayloadAction<ToolbarItem[]>) {
      if (state.items) {
        state.items.center = action.payload;
      }
    },
    setItemsOnRight(state, action: PayloadAction<ToolbarItem[]>) {
      if (state.items) {
        state.items.right = action.payload;
      }
    },
    addItem(state, action: PayloadAction<string>) {
      if (!state.items) {
        return;
      }
      const alreadyExists =
        state.items.left.includes(action.payload) ||
        state.items.right.includes(action.payload) ||
        state.items.center.includes(action.payload);
      if (!alreadyExists) {
        state.items.right.push(action.payload);
      }
    },
    removeItem(state, action: PayloadAction<string>) {
      let id = action.payload;
      if (state.items) {
        let filter = (d: any) => d !== id && d.id !== id;
        state.items.left = state.items.left.filter(filter);
        state.items.center = state.items.center.filter(filter);
        state.items.right = state.items.right.filter(filter);
      }
    },
  },
});

export const RootActions = RootSlice.actions;
export const Selectors = StateBuilder.compositeSelector(initialState);

export const selectDefaultOutput = createSelector(Selectors.mediaOutputs, (mediaOutputs) =>
  mediaOutputs.find((d) => d.is_default_multimedia),
);
