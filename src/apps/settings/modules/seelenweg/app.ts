import { parseAsCamel } from '../../../shared/schemas';
import { Seelenweg, SeelenWegSchema } from '../../../shared/schemas/Seelenweg';
import { createSlice } from '@reduxjs/toolkit';

import { reducersFor, selectorsFor } from '../shared/utils/app';

const initialState: Seelenweg = parseAsCamel(SeelenWegSchema, {});

export const SeelenWegSlice = createSlice({
  name: 'seelenweg',
  initialState,
  selectors: selectorsFor(initialState),
  reducers: reducersFor(initialState),
});

export const SeelenWegActions = SeelenWegSlice.actions;