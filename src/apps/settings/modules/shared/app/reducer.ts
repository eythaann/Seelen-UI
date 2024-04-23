import { matcher, reducersFor, selectorsFor } from './utils';
import { createSlice, PayloadAction } from '@reduxjs/toolkit';

import { AppsConfigSlice } from '../../appsConfigurations/app/reducer';
import { GeneralSettingsSlice } from '../../general/main/app';
import { MonitorsSlice } from '../../monitors/main/app';
import { SeelenWegSlice } from '../../seelenweg/app';
import { SeelenManagerSlice } from '../../WindowManager/main/app';

import { Route } from '../domain/routes';
import { RootState } from '../domain/state';

const initialState: RootState = {
  autostart: false,
  route: Route.GENERAL,
  generals: GeneralSettingsSlice.getInitialState(),
  seelenweg: SeelenWegSlice.getInitialState(),
  seelenwm: SeelenManagerSlice.getInitialState(),
  toBeSaved: false,
  monitors: MonitorsSlice.getInitialState(),
  appsConfigurations: AppsConfigSlice.getInitialState(),
  appsTemplates: [],
  ahkEnabled: true,
  updateNotification: false,
  availableThemes: [],
  theme: null,
};

export const RootSlice = createSlice({
  name: 'main',
  initialState,
  reducers: {
    ...reducersFor(initialState),
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
      .addMatcher(matcher(SeelenManagerSlice), (state, action) => {
        state.toBeSaved = true;
        state.seelenwm = SeelenManagerSlice.reducer(state.seelenwm, action);
      })
      .addMatcher(matcher(SeelenWegSlice), (state, action) => {
        state.toBeSaved = true;
        state.seelenweg = SeelenWegSlice.reducer(state.seelenweg, action);
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