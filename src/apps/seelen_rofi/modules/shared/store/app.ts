import { StateBuilder } from '../../../../shared/StateBuilder';
import { createSlice } from '@reduxjs/toolkit';
import { SeelenLauncherSettings, UIColors } from 'seelen-core';

import { LauncherState } from './domain';

const initialState: LauncherState = {
  colors: UIColors.default(),
  apps: [],
  settings: { ...new SeelenLauncherSettings() },
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
