import { combineSlices, createSlice } from '@reduxjs/toolkit';

import { GeneralSettingsSlice } from '../../general/main/app';

import { Route } from '../domain/routes';

const RouteSlice = createSlice({
  name: 'route',
  initialState: Route.GENERAL,
  reducers: {
    setRoute: (state, action) => action.payload,
  },
});

export const mainActions = {
  ...RouteSlice.actions,
};

export const mainReducer = combineSlices(RouteSlice, {
  general: GeneralSettingsSlice.reducer,
});