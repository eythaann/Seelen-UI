import { StateBuilder } from '../../../../shared/StateBuilder';
import { createSlice } from '@reduxjs/toolkit';

import { RootState } from './domain';

const initialState: RootState = {};

export const RootSlice = createSlice({
  name: 'root',
  initialState,
  reducers: {
    ...StateBuilder.reducersFor(initialState),
  },
});

export const Actions = RootSlice.actions;
export const Selectors = StateBuilder.compositeSelector(initialState);
