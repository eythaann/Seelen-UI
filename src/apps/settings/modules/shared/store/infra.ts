import { configureStore } from '@reduxjs/toolkit';
import { listen as listenGlobal } from '@tauri-apps/api/event';
import { Modal } from 'antd';
import { cloneDeep } from 'lodash';
import {
  AppConfiguration,
  ConnectedMonitorList,
  MonitorConfiguration,
  PluginList,
  ProfileList,
  SeelenEvent,
  Settings,
  Theme,
  UIColors,
  WidgetList,
} from 'seelen-core';

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
  const monitors = list.all();
  store.dispatch(RootActions.setConnectedMonitors(monitors));
  const settingsByMonitor = { ...store.getState().monitorsV2 };
  for (const monitor of monitors) {
    if (!settingsByMonitor[monitor.id]) {
      settingsByMonitor[monitor.id] = new MonitorConfiguration();
    }
  }
  console.log({ monitors, settingsByMonitor });
  store.dispatch(RootActions.setMonitorsV2(settingsByMonitor));
}

async function initUIColors() {
  function loadColors(colors: UIColors) {
    UIColors.setAssCssVariables(colors);
    store.dispatch(RootActions.setColors(colors));
  }
  loadColors(await UIColors.getAsync());
  await UIColors.onChange(loadColors);
}

export async function registerStoreEvents() {
  await listenGlobal<any[]>('placeholders', async () => {
    const userSettings = await new UserSettingsLoader().withPlaceholders().load();
    store.dispatch(RootActions.setAvailablePlaceholders(userSettings.placeholders));
  });

  await listenGlobal<Theme[]>('themes', (event) => {
    store.dispatch(RootActions.setAvailableThemes(event.payload));
  });

  await initUIColors();

  await listenGlobal<AppConfiguration[]>('settings-by-app', (event) => {
    store.dispatch(RootActions.setAppsConfigurations(event.payload));
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
  newState.lastLoaded = cloneDeep(newState);
  store.dispatch(RootActions.setState(newState));

  store.dispatch(RootActions.setPlugins((await PluginList.getAsync()).all()));
  store.dispatch(RootActions.setWidgets((await WidgetList.getAsync()).all()));
  store.dispatch(RootActions.setProfiles((await ProfileList.getAsync()).toArray()));
  setMonitorsOnState(await ConnectedMonitorList.getAsync());
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
