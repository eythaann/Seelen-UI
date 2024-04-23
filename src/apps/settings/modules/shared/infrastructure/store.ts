import { UserSettings } from '../../../../../shared.interfaces';
import { StartUser } from './StartUser';
import { loadAppsTemplates, loadUserSettings, saveUserSettings } from './storeApi';
import { startup } from './tauri';
import { configureStore } from '@reduxjs/toolkit';
import { emit } from '@tauri-apps/api/event';
import { Modal } from 'antd';

import { RootActions, RootReducer, RootSlice } from '../app/reducer';
import { StateAppsToYamlApps, StateToJsonSettings, StaticSettingsToState, YamlToState_Apps } from '../app/StateBridge';

import { RootState } from '../domain/state';

export const store = configureStore({
  reducer: RootReducer,
});

export type AppDispatch = typeof store.dispatch;
export type store = {
  dispatch: AppDispatch;
  getState: () => RootState;
};

export const LoadSettingsToStore = async (route?: string) => {
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

  const userSettings = await loadUserSettings(route);
  if (!Object.keys(userSettings.jsonSettings).length) {
    if (!route) { // avoid start user on manual user loading file
      StartUser();
    }
    return;
  }

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
      ahkEnabled: currentState.ahkEnabled,
      updateNotification: currentState.updateNotification,
      themes: currentState.availableThemes,
      theme: currentState.theme,
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
