import { createSlice } from '@reduxjs/toolkit';

import { reducersFor, selectorsFor } from '../../shared/app/utils';

import { BorderState } from './domain';

const initialState: BorderState = {
  enable: false,
  offset: -1,
  width: 8,
  colorSingle: '#ff0000',
  colorMonocle: '#00ff00',
  colorStack: '#0000ff',
};

export const BorderSlice = createSlice({
  name: 'generalSettings/border',
  initialState,
  reducers: reducersFor(initialState),
  selectors: selectorsFor(initialState),
});

export const BorderActions = BorderSlice.actions;