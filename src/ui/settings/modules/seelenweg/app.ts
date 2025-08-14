import { createSlice } from '@reduxjs/toolkit';
import { SeelenWegSettings } from '@seelen-ui/lib/types';

import { defaultSettings } from '../shared/store/app/default';
import { reducersFor, selectorsFor } from '../shared/utils/app';

const initialState: SeelenWegSettings = defaultSettings.seelenweg;

export const SeelenWegSlice = createSlice({
  name: 'seelenweg',
  initialState,
  selectors: selectorsFor(initialState),
  reducers: reducersFor(initialState),
});

export const SeelenWegActions = SeelenWegSlice.actions;