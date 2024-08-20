import { parseAsCamel } from '../../../../shared/schemas';
import { FancyToolbarSchema } from '../../../../shared/schemas/FancyToolbar';
import { Placeholder, ToolbarModule } from '../../../../shared/schemas/Placeholders';
import { StateBuilder } from '../../../../shared/StateBuilder';
import { createSlice, PayloadAction } from '@reduxjs/toolkit';

import { RootState } from './domain';

const initialState: RootState = {
  version: 0,
  focused: null,
  placeholder: null,
  settings: parseAsCamel(FancyToolbarSchema, {}),
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
  activeWorkspace: 0,
  systemTray: [],
  networkAdapters: [],
  networkLocalIp: null,
  online: false,
  wlanBssEntries: [],
  mediaSessions: [],
  mediaOutputs: [],
  mediaInputs: [],
  notifications: [],
  colors: {
    background: '#ffffff',
    foreground: '#000000',
    accent_darkest: '#000000',
    accent_darker: '#000000',
    accent_dark: '#000000',
    accent: '#000000',
    accent_light: '#000000',
    accent_lighter: '#000000',
    accent_lightest: '#000000',
    complement: null,
  },
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