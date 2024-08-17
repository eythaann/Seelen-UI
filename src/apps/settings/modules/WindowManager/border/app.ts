import { parseAsCamel } from '../../../../shared/schemas';
import { Border, BorderSchema } from '../../../../shared/schemas/WindowManager';
import { createSlice } from '@reduxjs/toolkit';

import { reducersFor, selectorsFor } from '../../shared/utils/app';

const initialState: Border = parseAsCamel(BorderSchema, {});

export const BorderSlice = createSlice({
  name: 'windowManager/border',
  initialState,
  reducers: reducersFor(initialState),
  selectors: selectorsFor(initialState),
});

export const BorderActions = BorderSlice.actions;