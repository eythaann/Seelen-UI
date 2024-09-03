import { StateBuilder } from '../shared/StateBuilder';
import { UIColors } from '../shared/styles';
import { createSlice } from '@reduxjs/toolkit';

interface RootState {
  colors: UIColors;
}

const initialState: RootState = {
  colors: UIColors.default(),
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
