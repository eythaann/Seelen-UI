import { createSlice } from '@reduxjs/toolkit';

import { BorderState } from './domain';

let initialState: BorderState = {
  enable: false,
  offset: null,
  width: 20,
  color: '#ff0000',
};

export const BorderSlice = createSlice({
  name: 'border',
  initialState,
  reducers: {
    toggleEnable: (state) => {
      state.enable = !state.enable;
    },
  },
});

export const BorderActions = BorderSlice.actions;