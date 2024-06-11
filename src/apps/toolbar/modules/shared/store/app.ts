import { defaultTheme } from '../../../../../shared.interfaces';
import { parseAsCamel } from '../../../../utils/schemas';
import { FancyToolbarSchema } from '../../../../utils/schemas/FancyToolbar';
import { StateBuilder } from '../../../../utils/StateBuilder';
import { createSlice } from '@reduxjs/toolkit';

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
};

export const RootSlice = createSlice({
  name: 'root',
  initialState,
  reducers: {
    ...StateBuilder.reducersFor(initialState),
  },
});

export const RootActions = RootSlice.actions;
export const Selectors = StateBuilder.compositeSelector(initialState);