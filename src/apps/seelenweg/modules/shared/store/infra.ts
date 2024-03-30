import { Theme } from '../../../../../shared.interfaces';
import { updateHitbox } from '../../../events';
import { configureStore } from '@reduxjs/toolkit';
import { path } from '@tauri-apps/api';
import { convertFileSrc, invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

import { loadUserSettings } from '../../../../settings/modules/shared/infrastructure/storeApi';
import { fs } from '../../../../settings/modules/shared/infrastructure/tauri';

import { JsonToState_Seelenweg } from '../../../../settings/modules/shared/app/StateBridge';
import { RootActions, RootSlice } from './app';

import { SeelenWegMode, SeelenWegState } from '../../../../settings/modules/seelenweg/domain';
import { AppFromBackground, HWND } from './domain';

export const store = configureStore({
  reducer: RootSlice.reducer,
});

export type AppDispatch = typeof store.dispatch;
export type store = {
  dispatch: AppDispatch;
  getState: () => {};
};

async function cleanItems(items: AppFromBackground[]) {
  const missingIcon = await path.resolve(
    await path.resourceDir(),
    'static',
    'icons',
    'missing.png',
  );

  const cleaned: AppFromBackground[] = [];

  for (const item of items) {
    if (!(await fs.exists(item.icon))) {
      item.icon = missingIcon;
    }
    item.icon = convertFileSrc(item.icon);
    cleaned.push(item);
  }

  return cleaned;
}

export async function registerStoreEvents() {
  const updateHitboxIfNeeded = () => {
    const { mode } = store.getState().settings;
    if (mode === SeelenWegMode.MIN_CONTENT) {
      updateHitbox();
    }
  };

  await listen<AppFromBackground[]>('update-store-apps', async (event) => {
    const items = await cleanItems(event.payload);
    items.forEach((item) => store.dispatch(RootActions.addOpenApp(item)));
    updateHitboxIfNeeded();
  });

  await listen<AppFromBackground>('add-open-app', async (event) => {
    const item = (await cleanItems([event.payload]))[0]!;
    store.dispatch(RootActions.addOpenApp(item));
    updateHitboxIfNeeded();
  });

  await listen<AppFromBackground>('update-open-app', async (event) => {
    const item = (await cleanItems([event.payload]))[0]!;
    store.dispatch(RootActions.updateOpenAppInfo(item));
    updateHitboxIfNeeded();
  });

  await listen<AppFromBackground>('replace-open-app', async (event) => {
    const item = (await cleanItems([event.payload]))[0]!;
    store.dispatch(RootActions.addOpenApp(item));
    store.dispatch(RootActions.removeOpenApp(item.process_hwnd));
  });

  await listen<HWND>('remove-open-app', (event) => {
    store.dispatch(RootActions.removeOpenApp(event.payload));
    updateHitboxIfNeeded();
  });

  await listen<SeelenWegState>('update-store-settings', (event) => {
    document.getElementById('root')!.style.margin = event.payload.margin + 'px';

    store.dispatch(RootActions.setSettings(event.payload));
    updateHitbox();
  });

  await listen<Theme>('update-store-theme', (event) => {
    store.dispatch(RootActions.setTheme(event.payload));
  });

  invoke('weg_request_apps');
}

export async function loadStore() {
  const userSettings = await loadUserSettings();
  const initialState = RootSlice.getInitialState();

  const settings = JsonToState_Seelenweg(userSettings.jsonSettings, initialState.settings);
  store.dispatch(RootActions.setSettings(settings));
  if (userSettings.theme) {
    store.dispatch(RootActions.setTheme(userSettings.theme));
  }
}
