import { StateBuilder } from '../../../../../shared/StateBuilder';
import { Route } from '../../../../components/navigation/routes';
import i18n from '../../../../i18n';
import { createSlice, PayloadAction } from '@reduxjs/toolkit';
import { cloneDeep, pick } from 'lodash';
import {
  SeelenLauncherSettings,
  SeelenWallSettings,
  SeelenWegSettings,
  UIColors,
  VirtualDesktopStrategy,
} from 'seelen-core';

import { AppsConfigSlice } from '../../../appsConfigurations/app/reducer';
import { FancyToolbarSlice } from '../../../fancyToolbar/app';
import { MonitorsSlice } from '../../../monitors/main/app';
import { SeelenWegSlice } from '../../../seelenweg/app';
import { AhkVariablesSlice } from '../../../shortcuts/app';
import { SeelenManagerSlice } from '../../../WindowManager/main/app';
import { matcher, reducersFor, selectorsFor } from '../../utils/app';

import { RootState } from '../domain';

const initialState: RootState = {
  lastLoaded: null,
  autostart: null,
  route: Route.GENERAL,
  fancyToolbar: FancyToolbarSlice.getInitialState(),
  seelenweg: new SeelenWegSettings(),
  wall: new SeelenWallSettings(),
  launcher: new SeelenLauncherSettings(),
  windowManager: SeelenManagerSlice.getInitialState(),
  toBeSaved: false,
  toBeRestarted: false,
  monitors: MonitorsSlice.getInitialState(),
  appsConfigurations: AppsConfigSlice.getInitialState(),
  ahkEnabled: true,
  ahkVariables: AhkVariablesSlice.getInitialState(),
  availableThemes: [],
  availableLayouts: [],
  availablePlaceholders: [],
  selectedTheme: [],
  devTools: false,
  language: navigator.language.split('-')[0] || 'en',
  colors: UIColors.default(),
  wallpaper: null,
  virtualDesktopStrategy: VirtualDesktopStrategy.Native,
  betaChannel: false,
};

export const RootSlice = createSlice({
  name: 'main',
  initialState,
  reducers: {
    ...reducersFor(initialState),
    setState: (_state, action: PayloadAction<RootState>) => {
      i18n.changeLanguage(action.payload.language);
      return action.payload;
    },
    setLanguage: (state, action: PayloadAction<string>) => {
      state.language = action.payload;
      state.toBeSaved = true;
      i18n.changeLanguage(action.payload);
    },
    setVirtualDesktopStrategy: (state, action: PayloadAction<VirtualDesktopStrategy>) => {
      state.toBeSaved = true;
      state.toBeRestarted = true;
      state.virtualDesktopStrategy = action.payload;
    },
    restoreToLastLoaded: (state) => {
      if (state.lastLoaded) {
        const toMaintain = pick(state, ['autostart', 'route', 'colors', 'lastLoaded']);
        const newState = {
          ...cloneDeep(state.lastLoaded),
          ...toMaintain,
        };
        i18n.changeLanguage(newState.language);
        return newState;
      }
      return state;
    },
    setDevTools: (state, action: PayloadAction<boolean>) => {
      state.toBeSaved = true;
      state.devTools = action.payload;
    },
    setBetaChannel: (state, action: PayloadAction<boolean>) => {
      state.toBeSaved = true;
      state.betaChannel = action.payload;
    },
    setSelectedTheme: (state, action: PayloadAction<RootState['selectedTheme']>) => {
      let themes = new Set(action.payload);
      if (!themes.has('default')) {
        themes.add('default');
      }
      state.toBeSaved = true;
      state.selectedTheme = Array.from(themes);
    },
    removeTheme: (state, action: PayloadAction<string>) => {
      state.toBeSaved = true;
      state.selectedTheme = state.selectedTheme.filter((x) => x !== action.payload);
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
