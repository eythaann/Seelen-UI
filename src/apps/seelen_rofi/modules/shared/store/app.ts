import { StateBuilder } from '../../../../shared/StateBuilder';
import { createSlice } from '@reduxjs/toolkit';
import { UIColors } from 'seelen-core';

import { LauncherState } from './domain';

const initialState: LauncherState = {
  colors: UIColors.default(),
  apps: [],
  settings: {},
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
