import { reducersFor, selectorsFor } from '../../shared/utils/app';
import { createSlice } from '@reduxjs/toolkit';

import { ContainerTabsState, ContainerTopBarMode } from './domain';

const initialState: ContainerTabsState = {
  mode: ContainerTopBarMode.ON_STACK,
};

export const ContainerTopBarSlice = createSlice({
  name: 'generalSettings/border',
  initialState,
  reducers: reducersFor(initialState),
  selectors: selectorsFor(initialState),
});

export const ContainerTopBarActions = ContainerTopBarSlice.actions;