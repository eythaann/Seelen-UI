import { createSlice } from '@reduxjs/toolkit';
import { Settings, UIColors } from '@seelen-ui/lib';

import { RootState } from './domain';

import { StateBuilder } from '../../../../shared/StateBuilder';

const initialState: RootState = {
  _version: 0,
  layout: null,
  settings: (await Settings.default()).inner.windowManager,
  colors: UIColors.default().inner,
  activeWindow: 0,
  reservation: null,
  overlayVisible: true,
};

export const RootSlice = createSlice({
  name: 'root',
  initialState,
  reducers: {
    ...StateBuilder.reducersFor(initialState),
    forceUpdate(state) {
      state._version += 1;
    },
  },
});

export const Actions = RootSlice.actions;
export const Selectors = StateBuilder.compositeSelector(initialState);
