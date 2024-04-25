import { createSlice } from '@reduxjs/toolkit';

import { reducersFor, selectorsFor } from '../../shared/utils/app';

import { BorderState } from './domain';

const initialState: BorderState = {
  enabled: true,
  offset: -1,
  width: 3,
};

export const BorderSlice = createSlice({
  name: 'generalSettings/border',
  initialState,
  reducers: reducersFor(initialState),
  selectors: selectorsFor(initialState),
});

export const BorderActions = BorderSlice.actions;