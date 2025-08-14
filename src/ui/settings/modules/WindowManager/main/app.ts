import { createSlice } from '@reduxjs/toolkit';
import { WindowManagerSettings } from '@seelen-ui/lib/types';

import { defaultSettings } from '../../shared/store/app/default';
import { matcher, reducersFor, selectorsFor } from '../../shared/utils/app';
import { BorderSlice } from '../border/app';

let initialState: WindowManagerSettings = defaultSettings.windowManager;

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
