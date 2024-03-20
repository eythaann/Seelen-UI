import { StateBuilder } from '../../../../utils/StateBuilder';
import { createSlice } from '@reduxjs/toolkit';

import { RootState } from './domain';

const initialState: RootState = {};

export const RootSlice = createSlice({
  name: 'root',
  initialState,
  reducers: StateBuilder.reducersFor(initialState),
});

export const Selectors = StateBuilder.compositeSelector(initialState);