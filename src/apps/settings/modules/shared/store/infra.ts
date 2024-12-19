import { configureStore } from '@reduxjs/toolkit';
import {
  AppConfigurationList,
  ConnectedMonitorList,
  PlaceholderList,
  PluginList,
  ProfileList,
  SeelenEvent,
  Settings,
  ThemeList,
  UIColors,
  WidgetList,
} from '@seelen-ui/lib';
import { listen as listenGlobal } from '@tauri-apps/api/event';
import { Modal } from 'antd';
import { cloneDeep } from 'lodash';

import { startup } from '../tauri/infra';

import { RootActions, RootReducer } from './app/reducer';
import { StateToJsonSettings, StaticSettingsToState } from './app/StateBridge';

import { RootState } from './domain';

import { saveUserSettings, UserSettingsLoader } from './storeApi';

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

function setMonitorsOnState(list: ConnectedMonitorList) {
  store.dispatch(RootActions.setConnectedMonitors(list.all()));
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

  const userSettings = await new UserSettingsLoader()
    .withLayouts()
    .withPlaceholders()
    .withUserApps()
    .withThemes()
    .withSystemWallpaper()
    .load(customPath);

  const currentState = store.getState();
  const newState = StaticSettingsToState(userSettings, currentState);
  store.dispatch(RootActions.setState(newState));

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
