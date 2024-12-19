import { createSlice } from '@reduxjs/toolkit';
import { Settings, UIColors } from '@seelen-ui/lib';

import { LauncherState } from './domain';

import { StateBuilder } from '../../../../shared/StateBuilder';

const initialState: LauncherState = {
  colors: UIColors.default().inner,
  apps: [],
  history: {},
  settings: (await Settings.default()).inner.launcher,
};

export const RootSlice = createSlice({
  name: 'root',
  initialState,
  reducers: {
    ...StateBuilder.reducersFor(initialState),
  },
});

export const Actions = RootSlice.actions;
export const Selectors = StateBuilder.compositeSelector(initialState);
