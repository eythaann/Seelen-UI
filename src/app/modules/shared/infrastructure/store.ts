import { UserSettings } from '../../../../shared.interfaces';
import { StartUser } from './StartUser';
import { configureStore } from '@reduxjs/toolkit';
import { Modal } from 'antd';

import { RootActions, RootReducer, RootSlice } from '../app/reducer';
import { StateToJsonSettings, StateToYamlSettings, StaticSettingsToState } from '../app/StateBridge';

import { RootState } from '../domain/state';

export const store = configureStore({
  reducer: RootReducer,
});

export type AppDispatch = typeof store.dispatch;
export type store = {
  dispatch: AppDispatch;
  getState: () => RootState;
};

export const LoadSettingsToStore = async () => {
  window.backgroundApi.getUserSettings().then((userSettings: UserSettings) => {
    if (!Object.keys(userSettings.jsonSettings)) {
      StartUser();
      return;
    }

    const currentState = store.getState();
    const initialState = RootSlice.getInitialState();

    store.dispatch(
      RootActions.setState({
        ...StaticSettingsToState(userSettings.jsonSettings, userSettings.yamlSettings, initialState),
        route: currentState.route,
      }),
    );
  });
};

export const SaveStore = async () => {
  try {
    const currentState = store.getState();
    await window.backgroundApi.saveUserSettings({
      jsonSettings: StateToJsonSettings(currentState),
      yamlSettings: StateToYamlSettings(currentState),
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
