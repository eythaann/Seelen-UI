import { configureStore } from '@reduxjs/toolkit';

import { RootActions, RootReducer, RootSlice } from '../app/reducer';
import { JsonToState } from '../app/StateBridge';

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
  window.backgroundApi.getUserSettings().then((userSettings) => {
    if (userSettings.jsonSettings) {
      store.dispatch(RootActions.setState(JsonToState(userSettings.jsonSettings, RootSlice.getInitialState())));
    }
  });
};

export const SaveStore = async () => {
  store.dispatch(RootActions.setSaved());
};