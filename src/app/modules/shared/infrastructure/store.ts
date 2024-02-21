import { UserSettings } from '../../../../shared.interfaces';
import { configureStore } from '@reduxjs/toolkit';
import { Modal } from 'antd';

import { RootActions, RootReducer, RootSlice } from '../app/reducer';
import { JsonToState, StateToJson } from '../app/StateBridge';

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
    const currentState = store.getState();
    const initialState = RootSlice.getInitialState();

    store.dispatch(
      RootActions.setState({
        ...JsonToState(userSettings.jsonSettings, userSettings.yamlSettings, initialState),
        route: currentState.route,
      }),
    );
  });
};

export const SaveStore = async () => {
  try {
    await window.backgroundApi.saveUserSettings({
      jsonSettings: StateToJson(store.getState()),
      yamlSettings: [],
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
