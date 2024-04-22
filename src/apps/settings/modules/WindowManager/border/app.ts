import { createSlice } from '@reduxjs/toolkit';

import { reducersFor, selectorsFor } from '../../shared/app/utils';

import { BorderState } from './domain';

const initialState: BorderState = {
  enabled: false,
  offset: -1,
  width: 8,
  color: '#ff0000',
  activeColor: '#00ff00',
};

export const BorderSlice = createSlice({
  name: 'generalSettings/border',
  initialState,
  reducers: reducersFor(initialState),
  selectors: selectorsFor(initialState),
});

export const BorderActions = BorderSlice.actions;