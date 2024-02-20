import { createSlice } from '@reduxjs/toolkit';

import { reducersFor, selectorsFor } from '../../shared/app/utils';

import { BorderState } from './domain';

const initialState: BorderState = {
  enable: false,
  offset: 0,
  width: 20,
  color: '#ff0000',
};

export const BorderSlice = createSlice({
  name: 'generalSettings/border',
  initialState,
  reducers: reducersFor(initialState),
  selectors: selectorsFor(initialState),
});

export const BorderActions = BorderSlice.actions;