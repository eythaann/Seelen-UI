import { matcher, selectorsFor } from './utils';
import { createSlice, PayloadAction } from '@reduxjs/toolkit';

import { AppsConfigSlice } from '../../appsConfigurations/app/reducer';
import { GeneralSettingsSlice } from '../../general/main/app';
import { MonitorsSlice } from '../../monitors/main/app';

import { Route } from '../domain/routes';
import { RootState } from '../domain/state';

const initialState: RootState = {
  route: Route.GENERAL,
  generals: GeneralSettingsSlice.getInitialState(),
  toBeSaved: false,
  monitors: MonitorsSlice.getInitialState(),
  appsConfigurations: AppsConfigSlice.getInitialState(),
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
    setState: (_state, action: PayloadAction<RootState>) => action.payload,
  },
  selectors: selectorsFor(initialState),
  extraReducers: (builder) => {
    builder
      .addMatcher(matcher(GeneralSettingsSlice), (state, action) => {
        state.toBeSaved = true;
        state.generals = GeneralSettingsSlice.reducer(state.generals, action);
      })
      .addMatcher(matcher(MonitorsSlice), (state, action) => {
        state.toBeSaved = true;
        state.monitors = MonitorsSlice.reducer(state.monitors, action);
      })
      .addMatcher(matcher(AppsConfigSlice), (state, action) => {
        state.toBeSaved = true;
        state.appsConfigurations = AppsConfigSlice.reducer(state.appsConfigurations, action);
      });
  },
});

export const RootActions = RootSlice.actions;
export const RootReducer = RootSlice.reducer;