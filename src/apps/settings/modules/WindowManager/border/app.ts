import { createSlice } from '@reduxjs/toolkit';
import { Border } from 'seelen-core';

import { reducersFor, selectorsFor } from '../../shared/utils/app';

const initialState = new Border();

export const BorderSlice = createSlice({
  name: 'windowManager/border',
  initialState,
  reducers: reducersFor(initialState),
  selectors: selectorsFor(initialState),
});

export const BorderActions = BorderSlice.actions;
