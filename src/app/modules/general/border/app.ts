import { createSlice } from '@reduxjs/toolkit';

import { reducersFor, selectorsFor } from '../../shared/app/utils';

import { BorderState } from './domain';

const initialState: BorderState = {
  enable: false,
  offset: 0,
  width: 20,
  colorSingle: '#cc0000',
  colorMonocle: '#00cc00',
  colorStack: '#0000cc',
};

export const BorderSlice = createSlice({
  name: 'generalSettings/border',
  initialState,
  reducers: reducersFor(initialState),
  selectors: selectorsFor(initialState),
});

export const BorderActions = BorderSlice.actions;