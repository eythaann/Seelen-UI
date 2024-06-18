import { parseAsCamel } from '../../../../shared/schemas';
import { WindowManager, WindowManagerSchema } from '../../../../shared/schemas/WindowManager';
import { createSlice } from '@reduxjs/toolkit';

import { matcher, reducersFor, selectorsFor } from '../../shared/utils/app';
import { BorderSlice } from '../border/app';

let initialState: WindowManager = parseAsCamel(WindowManagerSchema, {});

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
