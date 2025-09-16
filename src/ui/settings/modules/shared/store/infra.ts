import { configureStore } from "@reduxjs/toolkit";
import {
  AppConfigurationList,
  ConnectedMonitorList,
  IconPackList,
  MonitorConfiguration,
  PluginList,
  ProfileList,
  Settings,
  ThemeList,
  UIColors,
  WallpaperList,
  WidgetList,
} from "@seelen-ui/lib";
import { Modal } from "antd";
import { cloneDeep } from "lodash";

import { startup } from "../tauri/infra";

import { RootActions, RootReducer } from "./app/reducer";
import { StateToJsonSettings } from "./app/StateBridge";

import { RootState } from "./domain";

import { saveUserSettings } from "./storeApi";

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
  const monitors = { ...state.monitorsV3 };
  for (const item of list.asArray()) {
    if (!monitors[item.id]) {
      monitors[item.id] = cloneDeep(defaultMonitorConfig.inner);
    }
  }
  store.dispatch(RootActions.setConnectedMonitors(list.all()));
  store.dispatch(RootActions.setMonitorsV3(monitors));
}

async function initUIColors() {
  function loadColors(colors: UIColors) {
    colors.setAsCssVariables();
    store.dispatch(RootActions.setColors(colors.inner));
  }
  loadColors(await UIColors.getAsync());
  await UIColors.onChange(loadColors);
}

export async function registerStoreEvents() {
  ThemeList.onChange((list) => {
    store.dispatch(RootActions.setAvailableThemes(list.all()));
  });

  await initUIColors();

  AppConfigurationList.onChange((list) => {
    store.dispatch(RootActions.setAppsConfigurations(list.all()));
  });

  await Settings.onChange((settings) => {
    if (IsSavingSettings.current) {
      IsSavingSettings.current = false;
      return;
    }
    const currentState = store.getState();
    const newState: RootState = {
      ...currentState,
      ...settings.inner,
      toBeSaved: false,
      toBeRestarted: false,
      // migration since v2.1.0
      fancyToolbar: settings.fancyToolbar,
      windowManager: settings.windowManager,
      seelenweg: settings.seelenweg,
      wall: settings.wall,
      launcher: settings.launcher,
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

  await WallpaperList.onChange((list) => {
    store.dispatch(RootActions.setWallpapers(list.all()));
  });

  await ConnectedMonitorList.onChange(setMonitorsOnState);
}

export const LoadSettingsToStore = async (customPath?: string) => {
  const settings: Settings = customPath ? await Settings.loadCustom(customPath) : await Settings.getAsync();

  const currentState = store.getState();
  store.dispatch(
    RootActions.setState({
      ...currentState,
      ...settings.inner,
      // migration since v2.1.0
      fancyToolbar: settings.fancyToolbar,
      windowManager: settings.windowManager,
      seelenweg: settings.seelenweg,
      wall: settings.wall,
      launcher: settings.launcher,
    }),
  );

  store.dispatch(RootActions.setAutostart(await startup.isEnabled()));

  store.dispatch(
    RootActions.setAppsConfigurations(
      (await AppConfigurationList.getAsync()).all(),
    ),
  );

  store.dispatch(
    RootActions.setAvailableThemes((await ThemeList.getAsync()).all()),
  );
  store.dispatch(
    RootActions.setAvailableIconPacks((await IconPackList.getAsync()).all()),
  );

  store.dispatch(RootActions.setPlugins((await PluginList.getAsync()).all()));
  store.dispatch(RootActions.setWidgets((await WidgetList.getAsync()).all()));
  store.dispatch(RootActions.setProfiles((await ProfileList.getAsync()).all()));
  store.dispatch(
    RootActions.setWallpapers((await WallpaperList.getAsync()).all()),
  );

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
      title: "Error on Save",
      content: String(error),
      centered: true,
    });
  }
};
