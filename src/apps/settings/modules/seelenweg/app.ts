import { createSlice } from '@reduxjs/toolkit';
import { SeelenWegSettings } from 'seelen-core';

import { reducersFor, selectorsFor } from '../shared/utils/app';

const initialState = new SeelenWegSettings();

export const SeelenWegSlice = createSlice({
  name: 'seelenweg',
  initialState,
  selectors: selectorsFor(initialState),
  reducers: reducersFor(initialState),
});

export const SeelenWegActions = SeelenWegSlice.actions;