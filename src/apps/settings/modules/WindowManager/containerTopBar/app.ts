import { parseAsCamel } from '../../../../utils/schemas';
import { ContainerTabs, ContainerTabsSchema } from '../../../../utils/schemas/WindowManager';
import { createSlice } from '@reduxjs/toolkit';

import { reducersFor, selectorsFor } from '../../shared/utils/app';

const initialState: ContainerTabs = parseAsCamel(ContainerTabsSchema, {});

export const ContainerTopBarSlice = createSlice({
  name: 'generalSettings/border',
  initialState,
  reducers: reducersFor(initialState),
  selectors: selectorsFor(initialState),
});

export const ContainerTopBarActions = ContainerTopBarSlice.actions;