import { createSlice } from '@reduxjs/toolkit';
import { Settings, UIColors } from '@seelen-ui/lib';

import { RootState } from './domain';

import { StateBuilder } from '../../../../shared/StateBuilder';

const initialState: RootState = {
  version: 0,
  settings: (await Settings.default()).wall,
  colors: UIColors.default().inner,
  stop: false,
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
