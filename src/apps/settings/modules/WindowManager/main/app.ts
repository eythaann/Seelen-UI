import { createSlice } from '@reduxjs/toolkit';
import { WindowManagerSettings } from 'seelen-core';

import { matcher, reducersFor, selectorsFor } from '../../shared/utils/app';
import { BorderSlice } from '../border/app';

let initialState = new WindowManagerSettings();

export const SeelenManagerSlice = createSlice({
  name: 'windowManager',
  initialState,
  selectors: selectorsFor(initialState),
  reducers: {
    ...reducersFor(initialState),
  },
  extraReducers: (builder) => {
    builder
      .addMatcher(matcher(BorderSlice), (state, action) => {
        state.border = BorderSlice.reducer(state.border, action);
      });
  },
});

export const WManagerSettingsActions = SeelenManagerSlice.actions;
