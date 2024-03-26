import { createSlice } from '@reduxjs/toolkit';

import { reducersFor, selectorsFor } from '../shared/app/utils';

import { SeelenWegMode, SeelenWegState } from './domain';

const initialState: SeelenWegState = {
  enabled: true,
  mode: SeelenWegMode.MIN_CONTENT,
  size: 40,
  zoomSize: 70,
  margin: 8,
  padding: 8,
  spaceBetweenItems: 8,
};

export const SeelenWegSlice = createSlice({
  name: 'seelenweg',
  initialState,
  selectors: selectorsFor(initialState),
  reducers: reducersFor(initialState),
});

export const SeelenWegActions = SeelenWegSlice.actions;