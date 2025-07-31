import { createSlice, PayloadAction } from '@reduxjs/toolkit';
import { UIColors, UpdateChannel, VirtualDesktopStrategy } from '@seelen-ui/lib';
import {
  IconPackId,
  SeelenWallSettings,
  ThemeId,
  WallpaperId,
  WallpaperInstanceSettings,
  WidgetId,
} from '@seelen-ui/lib/types';
import { cloneDeep, pick } from 'lodash';

import { AppsConfigSlice } from '../../../appsConfigurations/app/reducer';
import { FancyToolbarSlice } from '../../../fancyToolbar/app';
import { SeelenWegSlice } from '../../../seelenweg/app';
import { SeelenManagerSlice } from '../../../WindowManager/main/app';
import { matcher, reducersFor, selectorsFor } from '../../utils/app';

import { RootState } from '../domain';

import { StateBuilder } from '../../../../../shared/StateBuilder';
import i18n from '../../../../i18n';
import { defaultSettings } from './default';

const initialState: RootState = {
  lastLoaded: null,
  autostart: null,
  shortcuts: defaultSettings.inner.shortcuts,
  fancyToolbar: FancyToolbarSlice.getInitialState(),
  seelenweg: defaultSettings.seelenweg,
  wall: defaultSettings.wall,
  launcher: defaultSettings.launcher,
  windowManager: SeelenManagerSlice.getInitialState(),
  toBeSaved: false,
  toBeRestarted: false,
  monitorsV3: {},
  connectedMonitors: [],
  appsConfigurations: AppsConfigSlice.getInitialState(),
  availableThemes: [],
  availableIconPacks: [],
  oldActiveThemes: [],
  activeIconPacks: [],
  activeThemes: [],
  devTools: false,
  drpc: true,
  language: navigator.language.split('-')[0] || 'en',
  dateFormat: 'ddd D MMM, hh:mm A',
  colors: UIColors.default().inner,
  virtualDesktopStrategy: VirtualDesktopStrategy.Native,
  updater: {
    channel: UpdateChannel.Release,
  },
  plugins: [],
  widgets: [],
  profiles: [],
  wallpapers: [],
  byWidget: defaultSettings.inner.byWidget,
  byTheme: {},
  byWallpaper: {},
};

function toBeSaved<S, A, R>(fn: (state: S, action: A) => R) {
  return (state: S, action: A) => {
    (state as RootState).toBeSaved = true;
    return fn(state, action);
  };
}

function toBeSavedAndRestarted<S, A, R>(fn: (state: S, action: A) => R) {
  return (state: S, action: A) => {
    (state as RootState).toBeSaved = true;
    (state as RootState).toBeRestarted = true;
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
      i18n.changeLanguage(action.payload.language || undefined);
      return action.payload;
    },
    setDateFormat: toBeSaved(reducers.setDateFormat),
    setWall: toBeSaved(reducers.setWall),
    setLauncher: toBeSaved(reducers.setLauncher),
    setDevTools: toBeSaved(reducers.setDevTools),
    setUpdater: toBeSavedAndRestarted(reducers.setUpdater),
    setDrpc: toBeSavedAndRestarted(reducers.setDrpc),
    setMonitors: toBeSaved(reducers.setMonitorsV3),
    setLanguage: (state, action: PayloadAction<string>) => {
      state.language = action.payload;
      state.toBeSaved = true;
      i18n.changeLanguage(action.payload);
    },
    setVirtualDesktopStrategy: toBeSavedAndRestarted(reducers.setVirtualDesktopStrategy),
    restoreToLastLoaded: (state) => {
      if (state.lastLoaded) {
        const toMaintain = pick(state, ['autostart', 'route', 'colors', 'lastLoaded']);
        const newState = {
          ...cloneDeep(state.lastLoaded),
          ...toMaintain,
        };
        i18n.changeLanguage(newState.language || undefined);
        return newState;
      }
      return state;
    },
    setActiveIconPacks: (state, action: PayloadAction<IconPackId[]>) => {
      let iconPacks = new Set(action.payload);
      // remove missing
      for (const id of action.payload) {
        if (!state.availableIconPacks.some((x) => x.id === id)) {
          iconPacks.delete(id);
        }
      }
      state.toBeSaved = true;
      state.activeIconPacks = Array.from(iconPacks);
    },
    setSelectedThemes: (state, action: PayloadAction<ThemeId[]>) => {
      let themes = new Set(action.payload);
      // remove missing
      for (const id of action.payload) {
        if (!state.availableThemes.some((x) => x.id === id)) {
          themes.delete(id);
        }
      }
      state.toBeSaved = true;
      state.activeThemes = Array.from(themes);
    },
    removeTheme: (state, action: PayloadAction<string>) => {
      state.toBeSaved = true;
      state.activeThemes = state.activeThemes.filter((x) => x !== action.payload);
    },
    patchWall: (state, action: PayloadAction<Partial<SeelenWallSettings>>) => {
      state.toBeSaved = true;
      state.wall = { ...state.wall, ...action.payload };
    },
    patchWallpaperSettings: (
      state,
      action: PayloadAction<{
        id: WallpaperId;
        patch: Partial<WallpaperInstanceSettings>;
      }>,
    ) => {
      const { id, patch } = action.payload;
      state.toBeSaved = true;
      state.byWallpaper[id] = {
        ...(state.byWallpaper[id] || {}),
        ...patch,
      } as WallpaperInstanceSettings;
    },
    resetWallpaperSettings: (state, action: PayloadAction<WallpaperId>) => {
      state.toBeSaved = true;
      delete state.byWallpaper[action.payload];
    },
    patchWidgetConfig(
      state,
      action: PayloadAction<{ widgetId: WidgetId; config: Record<string, unknown> }>,
    ) {
      const { widgetId, config } = action.payload;

      state.toBeSaved = true;
      state.byWidget[widgetId] ??= { enabled: true };
      state.byWidget[widgetId] = {
        ...state.byWidget[widgetId]!,
        ...config,
      };
    },
    patchWidgetInstanceConfig: (
      state,
      action: PayloadAction<{
        widgetId: WidgetId;
        instanceId: string;
        config: Record<string, any>;
      }>,
    ) => {
      const { widgetId, instanceId, config } = action.payload;

      state.toBeSaved = true;
      state.byWidget[widgetId] ??= { enabled: true };
      const widget = state.byWidget[widgetId]!;

      widget.$instances ??= {};
      widget.$instances[instanceId] ??= {};
      const instance = widget.$instances![instanceId]!;

      widget.$instances[instanceId] = {
        ...instance,
        ...config,
      };
    },
    patchWidgetMonitorConfig: (
      state,
      action: PayloadAction<{
        widgetId: WidgetId;
        monitorId: string;
        config: Record<string, any>;
      }>,
    ) => {
      const { widgetId, monitorId, config } = action.payload;

      let monitor = state.monitorsV3[monitorId];
      if (!monitor) {
        return;
      }

      state.toBeSaved = true;
      monitor.byWidget[widgetId] ??= { enabled: true };
      monitor.byWidget[widgetId] = {
        ...monitor.byWidget[widgetId]!,
        ...config,
      };
    },
    removeWidgetInstance: (
      state,
      action: PayloadAction<{ widgetId: WidgetId; instanceId: string }>,
    ) => {
      const { widgetId, instanceId } = action.payload;
      if (!state.byWidget[widgetId]) {
        return;
      }

      state.toBeSaved = true;
      const widget = state.byWidget[widgetId]!;
      delete widget.$instances?.[instanceId];

      if (Object.keys(widget.$instances || {}).length === 0) {
        delete widget.$instances;
      }
    },
    setThemeVariable: (
      state,
      action: PayloadAction<{ themeId: ThemeId; name: string; value: string }>,
    ) => {
      const { themeId, name, value } = action.payload;
      state.byTheme[themeId] ??= {};
      state.byTheme[themeId]![name] = value;
      state.toBeSaved = true;
    },
    deleteThemeVariable: (state, action: PayloadAction<{ themeId: ThemeId; name: string }>) => {
      const { themeId, name } = action.payload;
      state.byTheme[themeId] ??= {};
      delete state.byTheme[themeId]![name];
      state.toBeSaved = true;
    },
    resetThemeVariables: (state, action: PayloadAction<{ themeId: ThemeId }>) => {
      const { themeId } = action.payload;
      state.byTheme[themeId] = {};
      state.toBeSaved = true;
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
      });
  },
});

export const RootActions = RootSlice.actions;
export const RootReducer = RootSlice.reducer;

export const newSelectors = StateBuilder.compositeSelector(initialState);
