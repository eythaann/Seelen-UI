import { parseAsCamel } from '../../../utils/schemas';
import { FancyToolbar, FancyToolbarSchema } from '../../../utils/schemas/FancyToolbar';
import { createSlice } from '@reduxjs/toolkit';

import { reducersFor, selectorsFor } from '../shared/utils/app';

const initialState: FancyToolbar = parseAsCamel(FancyToolbarSchema, {});

export const FancyToolbarSlice = createSlice({
  name: 'fancyToolbar',
  initialState,
  selectors: selectorsFor(initialState),
  reducers: reducersFor(initialState),
});

export const FancyToolbarActions = FancyToolbarSlice.actions;