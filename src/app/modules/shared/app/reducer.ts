import { matcher, selectorsFor } from './utils';
import { createSlice, PayloadAction } from '@reduxjs/toolkit';

import { GeneralSettingsSlice } from '../../general/main/app';
import { MonitorsSlice } from '../../monitors/main/app';

import { Route } from '../domain/routes';
import { RootState } from '../domain/state';

const initialState: RootState = {
  route: Route.GENERAL,
  generals: GeneralSettingsSlice.getInitialState(),
  toBeSaved: false,
  monitors: MonitorsSlice.getInitialState(),
};

export const RootSlice = createSlice({
  name: 'main',
  initialState,
  reducers: {
    setRoute: (state, action: PayloadAction<Route>) => {
      state.route = action.payload;
    },
    setSaved: (state) => {
      state.toBeSaved = false;
    },
  },
  selectors: selectorsFor(initialState),
  extraReducers: (builder) => {
    builder
      .addMatcher(matcher(GeneralSettingsSlice), (state, action) => {
        state.toBeSaved = true;
        GeneralSettingsSlice.reducer(state.generals, action);
      })
      .addMatcher(matcher(MonitorsSlice), (state, action) => {
        state.toBeSaved = true;
        MonitorsSlice.reducer(state.monitors, action);
      });
  },
});

export const RootActions = RootSlice.actions;
export const RootReducer = RootSlice.reducer;