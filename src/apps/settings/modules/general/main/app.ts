import { createSlice } from '@reduxjs/toolkit';

import { reducersFor, selectorsFor } from '../../shared/app/utils';

import {
  GeneralSettingsState,
} from './domain';

let initialState: GeneralSettingsState = {
  selectedTheme: null,
};

export const GeneralSettingsSlice = createSlice({
  name: 'generalSettings',
  initialState,
  selectors: selectorsFor(initialState),
  reducers: {
    ...reducersFor(initialState),
  },
});

export const GeneralSettingsActions = GeneralSettingsSlice.actions;