import { Theme } from '../../../../../shared.interfaces';
import { updateHitbox } from '../../../events';
import { loadPinnedItems } from './storeApi';
import { configureStore } from '@reduxjs/toolkit';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

import { loadUserSettings } from '../../../../settings/modules/shared/infrastructure/storeApi';

import { JsonToState_Seelenweg } from '../../../../settings/modules/shared/app/StateBridge';
import { PinnedApp } from '../../item/app/PinnedApp';
import { TemporalApp } from '../../item/app/TemporalApp';
import { RootActions, RootSlice } from './app';

import { SeelenWegMode, SeelenWegState } from '../../../../settings/modules/seelenweg/domain';
import { AppFromBackground, HWND, SavedItems } from './domain';

export const store = configureStore({
  reducer: RootSlice.reducer,
});

export type AppDispatch = typeof store.dispatch;
export type store = {
  dispatch: AppDispatch;
  getState: () => {};
};

async function cleanItems(items: AppFromBackground[]): Promise<AppFromBackground[]> {
  const result: AppFromBackground[] = [];
  for (const item of items) {
    const cleaned = await TemporalApp.clean(item);
    result.push(cleaned);
  }
  return result;
}

async function cleanSavedItems(items: SavedItems[]): Promise<PinnedApp[]> {
  const result: PinnedApp[] = [];
  for (const item of items) {
    const cleaned = await PinnedApp.clean(item);
    result.push(PinnedApp.fromSaved(cleaned));
  }
  return result;
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
    loadCSSVariables(event.payload);
    store.dispatch(RootActions.setTheme(event.payload));
  });

  invoke('weg_request_apps');
}

function loadCSSVariables(theme: Theme) {
  Object.entries(theme.variables).forEach(([property, value]) => {
    document.documentElement.style.setProperty(property, value);
  });
}

export async function loadStore() {
  const userSettings = await loadUserSettings();
  const initialState = RootSlice.getInitialState();

  const settings = JsonToState_Seelenweg(userSettings.jsonSettings, initialState.settings);
  store.dispatch(RootActions.setSettings(settings));
  if (userSettings.theme) {
    loadCSSVariables(userSettings.theme);
    store.dispatch(RootActions.setTheme(userSettings.theme));
  }

  const apps = await loadPinnedItems();
  store.dispatch(RootActions.setPinnedOnLeft(await cleanSavedItems(apps.left) ));
  store.dispatch(RootActions.setPinnedOnCenter(await cleanSavedItems(apps.center )));
  store.dispatch(RootActions.setPinnedOnRight(await cleanSavedItems(apps.right )));
}
