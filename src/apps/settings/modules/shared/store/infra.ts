import { configureStore } from '@reduxjs/toolkit';
import {
  AppConfigurationList,
  ConnectedMonitorList,
  IconPackList,
  MonitorConfiguration,
  PlaceholderList,
  PluginList,
  ProfileList,
  SeelenEvent,
  Settings,
  ThemeList,
  UIColors,
  WidgetList,
  WindowManagerLayoutList,
} from '@seelen-ui/lib';
import { listen as listenGlobal } from '@tauri-apps/api/event';
import { Modal } from 'antd';
import { cloneDeep } from 'lodash';

import { startup } from '../tauri/infra';

import { RootActions, RootReducer } from './app/reducer';
import { StateToJsonSettings } from './app/StateBridge';

import { RootState } from './domain';

import { saveUserSettings } from './storeApi';

const IsSavingSettings = { current: false };

export const store = configureStore({
  reducer: RootReducer,
  middleware(getDefaultMiddleware) {
    return getDefaultMiddleware({
      serializableCheck: false,
    });
  },
});

export type AppDispatch = typeof store.dispatch;
export type store = {
  dispatch: AppDispatch;
  getState: () => RootState;
};

// ======================

const defaultMonitorConfig = await MonitorConfiguration.default();
function setMonitorsOnState(list: ConnectedMonitorList) {
  const state = store.getState();
  const monitors = { ...state.monitorsV2 };
  for (const item of list.asArray()) {
    if (!monitors[item.id]) {
      monitors[item.id] = cloneDeep(defaultMonitorConfig.inner);
    }
  }
  store.dispatch(RootActions.setConnectedMonitors(list.all()));
  store.dispatch(RootActions.setMonitorsV2(monitors));
}

async function initUIColors() {
  function loadColors(colors: UIColors) {
    colors.setAssCssVariables();
    store.dispatch(RootActions.setColors(colors.inner));
  }
  loadColors(await UIColors.getAsync());
  await UIColors.onChange(loadColors);
}

export async function registerStoreEvents() {
  PlaceholderList.onChange((list) => {
    store.dispatch(RootActions.setAvailablePlaceholders(list.all()));
  });

  WindowManagerLayoutList.onChange((list) => {
    store.dispatch(RootActions.setAvailableLayouts(list.all()));
  });

  ThemeList.onChange((list) => {
    store.dispatch(RootActions.setAvailableThemes(list.all()));
  });

  await initUIColors();

  AppConfigurationList.onChange((list) => {
    store.dispatch(RootActions.setAppsConfigurations(list.all()));
  });

  await listenGlobal<Settings>(SeelenEvent.StateSettingsChanged, (event) => {
    if (IsSavingSettings.current) {
      IsSavingSettings.current = false;
      return;
    }
    const currentState = store.getState();
    const newState: RootState = {
      ...currentState,
      ...event.payload,
      toBeSaved: false,
      toBeRestarted: false,
    };
    store.dispatch(RootActions.setState(newState));
  });

  await IconPackList.onChange((list) => {
    store.dispatch(RootActions.setAvailableIconPacks(list.all()));
  });

  await PluginList.onChange((list) => {
    store.dispatch(RootActions.setPlugins(list.all()));
  });

  await WidgetList.onChange((list) => {
    store.dispatch(RootActions.setWidgets(list.all()));
  });

  await ConnectedMonitorList.onChange(setMonitorsOnState);
}

export const LoadSettingsToStore = async (customPath?: string) => {
  startup.isEnabled().then((value) => {
    store.dispatch(RootActions.setAutostart(value));
  });

  const settings: Settings = customPath ? await Settings.loadCustom(customPath) : await Settings.getAsync();
  const currentState = store.getState();
  store.dispatch(RootActions.setState({
    ...currentState,
    ...settings.inner,
  }));

  store.dispatch(RootActions.setAppsConfigurations((await AppConfigurationList.getAsync()).all()));

  store.dispatch(RootActions.setAvailableThemes((await ThemeList.getAsync()).all()));
  store.dispatch(RootActions.setAvailableIconPacks((await IconPackList.getAsync()).all()));

  store.dispatch(RootActions.setAvailablePlaceholders((await PlaceholderList.getAsync()).all()));
  store.dispatch(RootActions.setAvailableLayouts((await WindowManagerLayoutList.getAsync()).all()));

  store.dispatch(RootActions.setPlugins((await PluginList.getAsync()).all()));
  store.dispatch(RootActions.setWidgets((await WidgetList.getAsync()).all()));
  store.dispatch(RootActions.setProfiles((await ProfileList.getAsync()).all()));

  setMonitorsOnState(await ConnectedMonitorList.getAsync());

  const state = { ...store.getState() };
  state.lastLoaded = cloneDeep(state);
  state.toBeSaved = false;
  store.dispatch(RootActions.setState(state));
};

export const SaveStore = async () => {
  try {
    const currentState = store.getState();
    const settings = {
      jsonSettings: StateToJsonSettings(currentState),
      yamlSettings: currentState.appsConfigurations,
    };

    IsSavingSettings.current = true;
    await saveUserSettings(settings);

    let newState = {
      ...currentState,
      lastLoaded: null,
      toBeSaved: false,
    };
    store.dispatch(
      RootActions.setState({
        ...newState,
        lastLoaded: cloneDeep(newState),
      }),
    );
  } catch (error) {
    Modal.error({
      title: 'Error on Save',
      content: String(error),
      centered: true,
    });
  }
};
