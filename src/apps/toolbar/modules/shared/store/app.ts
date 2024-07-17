import { defaultTheme } from '../../../../../shared.interfaces';
import { parseAsCamel } from '../../../../shared/schemas';
import { FancyToolbarSchema } from '../../../../shared/schemas/FancyToolbar';
import { ToolbarModule } from '../../../../shared/schemas/Placeholders';
import { StateBuilder } from '../../../../shared/StateBuilder';
import { createSlice, PayloadAction } from '@reduxjs/toolkit';

import { RootState } from './domain';

const initialState: RootState = {
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
  mediaVolume: 0,
  mediaMuted: false,
};

export const RootSlice = createSlice({
  name: 'root',
  initialState,
  reducers: {
    ...StateBuilder.reducersFor(initialState),
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