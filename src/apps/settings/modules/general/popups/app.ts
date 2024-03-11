
import { createSlice } from '@reduxjs/toolkit';

import { reducersFor, selectorsFor } from '../../shared/app/utils';

import { PopupState } from './domain';

const initialState: PopupState = {
  enable: true,
  x: null,
  y: null,
  height: 60,
  width: 280,
  textColor: '#1f1f1f',
  background: '#efefef',
  borderColor: '#333333',
  borderWidth: 0,
};

export const PopupSlice = createSlice({
  name: 'generalSettings/popups',
  initialState,
  reducers: reducersFor(initialState),
  selectors: selectorsFor(initialState),
});

export const PopupActions = PopupSlice.actions;