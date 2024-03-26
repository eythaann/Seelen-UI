import { Theme } from '../../../../../shared.interfaces';
import { configureStore } from '@reduxjs/toolkit';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

import { loadUserSettings } from '../../../../settings/modules/shared/infrastructure/storeApi';

import { JsonToState_Seelenweg } from '../../../../settings/modules/shared/app/StateBridge';
import { RootActions, RootSlice } from './app';

import { SeelenWegState } from '../../../../settings/modules/seelenweg/domain';
import { OpenApp } from './domain';

export const store = configureStore({
  reducer: RootSlice.reducer,
});

export type AppDispatch = typeof store.dispatch;
export type store = {
  dispatch: AppDispatch;
  getState: () => {};
};

export async function registerStoreEvents() {
  await listen<any[]>('update-store-apps', (event) => {
    store.dispatch(
      RootActions.setApps(
        event.payload.map<OpenApp>((app) => {
          return {
            ...app,
            state: 'Open',
          };
        }),
      ),
    );
  });

  await listen<SeelenWegState>('update-store-settings', (event) => {
    document.getElementById('root')!.style.margin = event.payload.margin + 'px';
    store.dispatch(RootActions.setSettings(event.payload));
  });

  await listen<Theme>('update-store-theme', (event) => {
    store.dispatch(RootActions.setTheme(event.payload));
  });

  invoke('weg_request_apps');
}

export async function loadStore() {
  const userSettings = await loadUserSettings();
  const initialState = RootSlice.getInitialState();

  store.dispatch(
    RootActions.setSettings(JsonToState_Seelenweg(userSettings.jsonSettings, initialState.settings)),
  );
  store.dispatch(RootActions.setTheme(userSettings.theme));
}
