import { createSlice } from '@reduxjs/toolkit';
import { FancyToolbarSettings } from 'seelen-core';

import { reducersFor } from '../shared/utils/app';

const initialState = new FancyToolbarSettings();

export const FancyToolbarSlice = createSlice({
  name: 'fancyToolbar',
  initialState,
  reducers: reducersFor(initialState),
});

export const FancyToolbarActions = FancyToolbarSlice.actions;