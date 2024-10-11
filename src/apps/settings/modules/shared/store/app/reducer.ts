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
import { SeelenWegSlice } from '../../../seelenweg/app';
import { AhkVariablesSlice } from '../../../shortcuts/app';
import { SeelenManagerSlice } from '../../../WindowManager/main/app';
import { matcher, reducersFor, selectorsFor } from '../../utils/app';

import { RootState } from '../domain';

import { StateBuilder } from '../../../../../shared/StateBuilder';
import { Route } from '../../../../components/navigation/routes';
import i18n from '../../../../i18n';

const initialState: RootState = {
  lastLoaded: null,
  autostart: null,
  route: Route.HOME,
  fancyToolbar: FancyToolbarSlice.getInitialState(),
  seelenweg: new SeelenWegSettings(),
  wall: new SeelenWallSettings(),
  launcher: new SeelenLauncherSettings(),
  windowManager: SeelenManagerSlice.getInitialState(),
  toBeSaved: false,
  toBeRestarted: false,
  monitors: [],
  appsConfigurations: AppsConfigSlice.getInitialState(),
  ahkEnabled: true,
  ahkVariables: AhkVariablesSlice.getInitialState(),
  availableThemes: [],
  availableLayouts: [],
  availablePlaceholders: [],
  iconPacks: [],
  selectedThemes: [],
  devTools: false,
  language: navigator.language.split('-')[0] || 'en',
  dateFormat: 'ddd D MMM, hh:mm A',
  colors: UIColors.default(),
  wallpaper: null,
  virtualDesktopStrategy: VirtualDesktopStrategy.Native,
  betaChannel: false,
};

function toBeSaved<S, A, R>(fn: (state: S, action: A) => R) {
  return (state: S, action: A) => {
    (state as RootState).toBeSaved = true;
    return fn(state, action);
  };
}

const reducers = reducersFor(initialState);
export const RootSlice = createSlice({
  name: 'main',
  initialState,
  reducers: {
    ...reducers,
    setState: (_state, action: PayloadAction<RootState>) => {
      i18n.changeLanguage(action.payload.language);
      return action.payload;
    },
    setDateFormat: toBeSaved(reducers.setDateFormat),
    setWall: toBeSaved(reducers.setWall),
    setLauncher: toBeSaved(reducers.setLauncher),
    setDevTools: toBeSaved(reducers.setDevTools),
    setBetaChannel: toBeSaved(reducers.setBetaChannel),
    setMonitors: toBeSaved(reducers.setMonitors),
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
    setSelectedThemes: (state, action: PayloadAction<string[]>) => {
      let themes = new Set(action.payload);
      if (!themes.has('default')) {
        themes.add('default');
      }
      state.toBeSaved = true;
      state.selectedThemes = Array.from(themes);
    },
    removeTheme: (state, action: PayloadAction<string>) => {
      state.toBeSaved = true;
      state.selectedThemes = state.selectedThemes.filter((x) => x !== action.payload);
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
