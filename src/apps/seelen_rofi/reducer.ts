import { IRootState } from '../../shared.interfaces';
import { StateBuilder } from '../shared/StateBuilder';
import { createSlice } from '@reduxjs/toolkit';
import { UIColors } from 'seelen-core';

interface RootState extends IRootState<{}> {}

const initialState: RootState = {
  colors: UIColors.default(),
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
