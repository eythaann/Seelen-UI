import { defaultTheme } from '../../../../../shared.interfaces';
import { parseAsCamel } from '../../../../utils/schemas';
import { FancyToolbarSchema } from '../../../../utils/schemas/FancyToolbar';
import { StateBuilder } from '../../../../utils/StateBuilder';
import { createSlice } from '@reduxjs/toolkit';

import { RootState } from './domain';

const initialState: RootState = {
  focused: null,
  theme: defaultTheme,
  settings: parseAsCamel(FancyToolbarSchema, {}),
};

export const RootSlice = createSlice({
  name: 'root',
  initialState,
  reducers: {
    ...StateBuilder.reducersFor(initialState),
  },
});

export const RootActions = RootSlice.actions;
export const Selectors = StateBuilder.compositeSelector(initialState);