import { createSlice } from '@reduxjs/toolkit';
import { Settings, UIColors } from '@seelen-ui/lib';
import { StateBuilder } from '@shared/StateBuilder';

import { RootState } from './domain';

const initialState: RootState = {
  settings: (await Settings.default()).windowManager,
  colors: UIColors.default().inner,
  reservation: null,
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
