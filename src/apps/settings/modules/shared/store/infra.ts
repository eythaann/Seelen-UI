import { UserSettings } from '../../../../../shared.interfaces';
import { setColorsAsCssVariables } from '../../../../shared';
import {
  getBackgroundLayers,
  loadAppsTemplates,
  saveUserSettings,
  UserSettingsLoader,
} from './storeApi';
import { configureStore } from '@reduxjs/toolkit';
import { invoke } from '@tauri-apps/api/core';
import { emit, listen as listenGlobal } from '@tauri-apps/api/event';
import { Modal } from 'antd';
import { cloneDeep } from 'lodash';

import { StartUser } from '../../StartUser/infra';
import { startup } from '../tauri/infra';

import { RootActions, RootReducer, RootSlice } from './app/reducer';
import {
  StateAppsToYamlApps,
  StateToJsonSettings,
  StaticSettingsToState,
  YamlToState_Apps,
} from './app/StateBridge';

import { RootState, UIColors } from './domain';

export const store = configureStore({
  reducer: RootReducer,
});

export type AppDispatch = typeof store.dispatch;
export type store = {
  dispatch: AppDispatch;
  getState: () => RootState;
};

export async function registerStoreEvents() {
  await listenGlobal<UIColors>('colors', (event) => {
    setColorsAsCssVariables(event.payload);
    store.dispatch(RootActions.setColors(event.payload));
  });
}

export const LoadSettingsToStore = async (customPath?: string) => {
  startup.isEnabled().then((value) => {
    store.dispatch(RootActions.setAutostart(value));
  });

  const appsTemplate = await loadAppsTemplates();
  store.dispatch(
    RootActions.setAppsTemplates(
      appsTemplate.map((template) => {
        return {
          ...template,
          apps: YamlToState_Apps(template.apps),
        };
      }),
    ),
  );

  const userSettings = await new UserSettingsLoader()
    .withLayouts()
    .withPlaceholders()
    .withUserApps()
    .load(customPath);

  const currentState = store.getState();
  const loadedState = StaticSettingsToState(userSettings, RootSlice.getInitialState());

  let state = {
    ...loadedState,
    appsTemplates: currentState.appsTemplates,
    route: currentState.route,
    autostart: currentState.autostart,
    colors: currentState.colors,
  };
  state.lastLoaded = cloneDeep(state);

  store.dispatch(RootActions.setState(state));

  // !customPath => avoid start user on manual user loading file
  if (!Object.keys(userSettings.jsonSettings).length && !customPath) {
    StartUser();
  }
};

export const SaveStore = async () => {
  try {
    const currentState = store.getState();
    const settings: UserSettings = {
      jsonSettings: StateToJsonSettings(currentState),
      yamlSettings: [
        //...StateAppsToYamlApps(currentState.appsTemplates.flatMap((x) => x.apps), true),
        ...StateAppsToYamlApps(currentState.appsConfigurations),
      ],
      themes: currentState.availableThemes,
      bgLayers: getBackgroundLayers(
        [currentState.selectedTheme].flat(),
        currentState.availableThemes,
      ),
      layouts: currentState.availableLayouts,
      placeholders: currentState.availablePlaceholders,
      env: await invoke('get_user_envs'),
    };

    await saveUserSettings(settings);
    await emit('updated-settings', settings);

    store.dispatch(RootActions.setToBeSaved(false));
  } catch (error) {
    Modal.error({
      title: 'Error on Save',
      content: String(error),
      centered: true,
    });
  }
};
