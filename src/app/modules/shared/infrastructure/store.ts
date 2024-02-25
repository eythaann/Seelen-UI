import { StartUser } from './StartUser';
import { configureStore } from '@reduxjs/toolkit';
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
  const appsTemplate = await window.backgroundApi.loadAppsTemplates();
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

  const userSettings = await window.backgroundApi.getUserSettings(route);
  if (!Object.keys(userSettings.jsonSettings).length) {
    StartUser();
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
    }),
  );
};

export const SaveStore = async () => {
  try {
    const currentState = store.getState();
    await window.backgroundApi.saveUserSettings({
      jsonSettings: StateToJsonSettings(currentState),
      yamlSettings: [
        ...StateAppsToYamlApps(currentState.appsTemplates.flatMap((x) => x.apps), true),
        ...StateAppsToYamlApps(currentState.appsConfigurations),
      ],
      ahkEnabled: currentState.ahkEnabled,
    });
    store.dispatch(RootActions.setSaved());
  } catch (error) {
    Modal.error({
      title: 'Error on Save',
      content: String(error),
      centered: true,
    });
  }
};
