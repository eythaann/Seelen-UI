import { UserSettings } from '../../../../../shared.interfaces';
import { loadAppsTemplates, loadUserSettings, saveUserSettings } from './storeApi';
import { configureStore } from '@reduxjs/toolkit';
import { invoke } from '@tauri-apps/api/core';
import { emit } from '@tauri-apps/api/event';
import { Modal } from 'antd';

import { StartUser } from '../../StartUser/infra';
import { startup } from '../tauri/infra';

import { RootActions, RootReducer, RootSlice } from './app/reducer';
import { StateAppsToYamlApps, StateToJsonSettings, StaticSettingsToState, YamlToState_Apps } from './app/StateBridge';

import { RootState } from './domain';

export const store = configureStore({
  reducer: RootReducer,
});

export type AppDispatch = typeof store.dispatch;
export type store = {
  dispatch: AppDispatch;
  getState: () => RootState;
};

export const LoadSettingsToStore = async (customPath?: string) => {
  store.dispatch(RootActions.setAutostart(await startup.isEnabled()));

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

  const userSettings = await loadUserSettings(customPath);

  const currentState = store.getState();
  const initialState = RootSlice.getInitialState();
  const loadedStore = StaticSettingsToState(userSettings, initialState);

  store.dispatch(
    RootActions.setState({
      ...loadedStore,
      appsTemplates: currentState.appsTemplates,
      route: currentState.route,
      autostart: currentState.autostart,
    }),
  );

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
        ...StateAppsToYamlApps(currentState.appsTemplates.flatMap((x) => x.apps), true),
        ...StateAppsToYamlApps(currentState.appsConfigurations),
      ],
      themes: currentState.availableThemes,
      theme: currentState.availableThemes.find((t) => t.info.filename === currentState.selectedTheme) || null,
      layouts: currentState.availableLayouts,
      placeholders: currentState.availablePlaceholders,
      env: await invoke('get_user_envs'),
    };

    await saveUserSettings(settings);
    await emit('updated-settings', settings);

    store.dispatch(RootActions.setSaved());
  } catch (error) {
    Modal.error({
      title: 'Error on Save',
      content: String(error),
      centered: true,
    });
  }
};
