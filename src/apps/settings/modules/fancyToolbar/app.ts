import { createSlice } from '@reduxjs/toolkit';

import { defaultSettings } from '../shared/store/app/default';
import { reducersFor } from '../shared/utils/app';

const initialState = defaultSettings.inner.fancyToolbar;

export const FancyToolbarSlice = createSlice({
  name: 'fancyToolbar',
  initialState,
  reducers: reducersFor(initialState),
});

export const FancyToolbarActions = FancyToolbarSlice.actions;