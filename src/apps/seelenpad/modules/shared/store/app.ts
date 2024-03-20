import { StateBuilder } from '../../../../utils/StateBuilder';
import { createSlice } from '@reduxjs/toolkit';

import { RouletteSlice } from '../../roulette/app/slice';

import { RootState } from './domain';

const initialState: RootState = StateBuilder.compositeInitialState(RouletteSlice);

export const RootSlice = createSlice({
  name: 'root',
  initialState,
  reducers: StateBuilder.reducersFor(initialState),
  extraReducers(builder) {
    StateBuilder.addSliceAsExtraReducer(RouletteSlice, builder);
  },
});

export const Selectors = StateBuilder.compositeSelector(initialState);