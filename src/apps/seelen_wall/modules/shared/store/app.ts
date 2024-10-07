import { createSlice } from '@reduxjs/toolkit';
import { SeelenWallSettings, UIColors } from 'seelen-core';

import { RootState } from './domain';

import { StateBuilder } from '../../../../shared/StateBuilder';

const initialState: RootState = {
  settings: new SeelenWallSettings(),
  colors: UIColors.default(),
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
