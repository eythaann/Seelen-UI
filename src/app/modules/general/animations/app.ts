import { createSlice, PayloadAction } from '@reduxjs/toolkit';

import { selectorsFor } from '../../shared/app/utils';

import { AnimationsState } from './domain';

let initialState: AnimationsState = {
  finishMiminization: true,
  nativeDelay: 35,
};

export const AnimationsSlice = createSlice({
  name: 'generalSettings/animations',
  initialState,
  selectors: selectorsFor(initialState),
  reducers: {
    toggleWaitMinimization: (state) => {
      state.finishMiminization = !state.finishMiminization;
    },
    setNativeDelay: (state, action: PayloadAction<number>) => {
      state.nativeDelay = action.payload;
    },
  },
});

export const AnimationsActions = AnimationsSlice.actions;