import { createSlice } from '@reduxjs/toolkit';

import { Rect } from '../../shared/app/Rect';
import { matcher, reducersFor, selectorsFor } from '../../shared/app/utils';
import { BorderSlice } from '../border/app';
import { ContainerTopBarSlice } from '../containerTopBar/app';

import {
  SeelenManagerState,
} from './domain';

let initialState: SeelenManagerState = {
  autoStackinByCategory: true,
  border: BorderSlice.getInitialState(),
  containerPadding: 10,
  workspacePadding: 10,
  resizeDelta: 50,
  globalWorkAreaOffset: new Rect().toJSON(),
  containerTopBar: ContainerTopBarSlice.getInitialState(),
};

export const WManagerSlice = createSlice({
  name: 'generalSettings',
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

export const WManagerSettingsActions = WManagerSlice.actions;
