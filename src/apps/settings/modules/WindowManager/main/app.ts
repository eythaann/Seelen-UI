import { createSlice } from '@reduxjs/toolkit';

import { Rect } from '../../shared/utils/app/Rect';
import { matcher, reducersFor, selectorsFor } from '../../shared/utils/app';
import { BorderSlice } from '../border/app';
import { ContainerTopBarSlice } from '../containerTopBar/app';

import {
  SeelenManagerState,
} from './domain';

let initialState: SeelenManagerState = {
  enable: true,
  autoStackinByCategory: true,
  border: BorderSlice.getInitialState(),
  containerPadding: 10,
  workspacePadding: 10,
  resizeDelta: 10,
  globalWorkAreaOffset: new Rect().toJSON(),
  containerTopBar: ContainerTopBarSlice.getInitialState(),
  floating: {
    width: 800,
    height: 500,
  },
};

export const SeelenManagerSlice = createSlice({
  name: 'seelenManagerSettings',
  initialState,
  selectors: selectorsFor(initialState),
  reducers: {
    ...reducersFor(initialState),
  },
  extraReducers: (builder) => {
    builder
      .addMatcher(matcher(BorderSlice), (state, action) => {
        state.border = BorderSlice.reducer(state.border, action);
      })
      .addMatcher(matcher(ContainerTopBarSlice), (state, action) => {
        state.containerTopBar = ContainerTopBarSlice.reducer(state.containerTopBar, action);
      });
  },
});

export const WManagerSettingsActions = SeelenManagerSlice.actions;
