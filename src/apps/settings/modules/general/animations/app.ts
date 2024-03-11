import { createSlice } from '@reduxjs/toolkit';

import { reducersFor, selectorsFor } from '../../shared/app/utils';

import { AnimationsState } from './domain';

let initialState: AnimationsState = {
  finishMiminization: true,
  nativeDelay: 35,
};

export const AnimationsSlice = createSlice({
  name: 'generalSettings/animations',
  initialState,
  selectors: selectorsFor(initialState),
  reducers: reducersFor(initialState),
});

export const AnimationsActions = AnimationsSlice.actions;