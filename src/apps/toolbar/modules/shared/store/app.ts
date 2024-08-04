import { defaultTheme } from '../../../../../shared.interfaces';
import { parseAsCamel } from '../../../../shared/schemas';
import { FancyToolbarSchema } from '../../../../shared/schemas/FancyToolbar';
import { Placeholder, ToolbarModule } from '../../../../shared/schemas/Placeholders';
import { StateBuilder } from '../../../../shared/StateBuilder';
import { createSlice, PayloadAction } from '@reduxjs/toolkit';

import { RootState } from './domain';

const initialState: RootState = {
  version: 0,
  focused: null,
  themeLayers: defaultTheme.layers,
  placeholder: null,
  settings: parseAsCamel(FancyToolbarSchema, {}),
  env: {},
  // default values of https://learn.microsoft.com/en-us/windows/win32/api/winbase/ns-winbase-system_power_status
  powerStatus: {
    ACLineStatus: 255,
    BatteryFlag: 255,
    BatteryLifePercent: 255,
    SystemStatusFlag: 0,
    BatteryLifeTime: -1,
    BatteryFullLifeTime: -1,
  },
  batteries: [],
  workspaces: [],
  activeWorkspace: 0,
  systemTray: [],
  networkAdapters: [],
  networkLocalIp: null,
  online: false,
  accentColor: '#ff0000',
  wlanBssEntries: [],
  mediaSessions: [],
  mediaOutputs: [],
  mediaInputs: [],
  notifications: [],
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