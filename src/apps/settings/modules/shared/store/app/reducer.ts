import { StateBuilder } from '../../../../../utils/StateBuilder';
import { Route } from '../../../../components/navigation/routes';
import { createSlice, PayloadAction } from '@reduxjs/toolkit';

import { AppsConfigSlice } from '../../../appsConfigurations/app/reducer';
import { FancyToolbarSlice } from '../../../fancyToolbar/app';
import { MonitorsSlice } from '../../../monitors/main/app';
import { SeelenWegSlice } from '../../../seelenweg/app';
import { AhkVariablesSlice } from '../../../shortcuts/app';
import { SeelenManagerSlice } from '../../../WindowManager/main/app';
import { matcher, reducersFor, selectorsFor } from '../../utils/app';

import { RootState } from '../domain';

const initialState: RootState = {
  autostart: false,
  route: Route.GENERAL,
  fancyToolbar: FancyToolbarSlice.getInitialState(),
  seelenweg: SeelenWegSlice.getInitialState(),
  windowManager: SeelenManagerSlice.getInitialState(),
  toBeSaved: false,
  monitors: MonitorsSlice.getInitialState(),
  appsConfigurations: AppsConfigSlice.getInitialState(),
  appsTemplates: [],
  ahkEnabled: true,
  ahkVariables: AhkVariablesSlice.getInitialState(),
  availableThemes: [],
  availableLayouts: [],
  availablePlaceholders: [],
  selectedTheme: null,
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
    setSelectedTheme: (state, action: PayloadAction<string>) => {
      state.toBeSaved = true;
      state.selectedTheme = action.payload;
    },
  },
  selectors: selectorsFor(initialState),
  extraReducers: (builder) => {
    builder
      .addMatcher(matcher(SeelenManagerSlice), (state, action) => {
        state.toBeSaved = true;
        state.windowManager = SeelenManagerSlice.reducer(state.windowManager, action);
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
      })
      .addMatcher(matcher(FancyToolbarSlice), (state, action) => {
        state.toBeSaved = true;
        state.fancyToolbar = FancyToolbarSlice.reducer(state.fancyToolbar, action);
      })
      .addMatcher(matcher(AhkVariablesSlice), (state, action) => {
        state.toBeSaved = true;
        state.ahkVariables = AhkVariablesSlice.reducer(state.ahkVariables, action);
      });
  },
});

export const RootActions = RootSlice.actions;
export const RootReducer = RootSlice.reducer;

export const newSelectors = StateBuilder.compositeSelector(initialState);