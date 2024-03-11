import { createSlice } from '@reduxjs/toolkit';

import { reducersFor, selectorsFor } from '../../shared/app/utils';

import { ContainerTabsState, ContainerTopBarMode } from './domain';

const initialState: ContainerTabsState = {
  mode: ContainerTopBarMode.ON_STACK,
  height: 40,
  tabs: {
    width: 200,
    color: '#efefef',
    background: '#333333',
  },
};

export const ContainerTopBarSlice = createSlice({
  name: 'generalSettings/border',
  initialState,
  reducers: reducersFor(initialState),
  selectors: selectorsFor(initialState),
});

export const ContainerTopBarActions = ContainerTopBarSlice.actions;