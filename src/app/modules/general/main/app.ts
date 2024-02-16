import { createSlice } from '@reduxjs/toolkit';

import { AnimationsSlice } from '../animations/app';
import { BorderSlice } from '../border/app';

import { GeneralSettingsState } from './domain';

let initialState: GeneralSettingsState = {
  altFocusHack: false,
  autoStackinByCategory: true,
  animations: AnimationsSlice.getInitialState(),
  border: BorderSlice.getInitialState(),
};

export const GeneralSettingsSlice = createSlice({
  name: 'GeneralSettings',
  initialState,
  reducers: {
    toggleAltFocuseHack: (state) => {
      state.altFocusHack = !state.altFocusHack;
    },
    toggleAutoStackinByCategory: (state) => {
      state.autoStackinByCategory = !state.autoStackinByCategory;
    },
  },
  extraReducers: (builder) => {
    builder
      .addCase(BorderSlice.name, (state, action) => {
        BorderSlice.reducer(state.border, action);
      })
      .addCase(AnimationsSlice.name, (state, action) => {
        AnimationsSlice.reducer(state.animations, action);
      });
  },
});

export const GeneralSettingsActions = GeneralSettingsSlice.actions;