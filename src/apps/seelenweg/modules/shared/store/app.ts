import { StateBuilder } from '../../../../utils/StateBuilder';
import { createSlice } from '@reduxjs/toolkit';

import { SeelenWegSlice } from '../../bar/app';

import { RootState } from './domain';

const initialState: RootState = {
  apps: [],
  pinnedOnLeft: [],
  pinnedOnCenter: [],
  pinnedOnRight: [],
  theme: null,
  settings: SeelenWegSlice.getInitialState(),
};

export const RootSlice = createSlice({
  name: 'root',
  initialState,
  reducers: StateBuilder.reducersFor(initialState),
});

export const RootActions = RootSlice.actions;
export const Selectors = StateBuilder.compositeSelector(initialState);