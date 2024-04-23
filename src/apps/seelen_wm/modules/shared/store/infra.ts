import { Theme } from '../../../../../shared.interfaces';
import { loadThemeCSS } from '../../../../utils';
import { configureStore } from '@reduxjs/toolkit';
import { listen } from '@tauri-apps/api/event';

import { loadUserSettings } from '../../../../settings/modules/shared/infrastructure/storeApi';

import { JsonToState_WManager } from '../../../../settings/modules/shared/app/StateBridge';
import { RootActions, RootSlice } from './app';

import { SeelenManagerState } from '../../../../settings/modules/WindowManager/main/domain';
import { Reservation, Sizing } from '../../layout/domain';
import { HWND } from '../utils/domain';
import { DesktopId, FocusAction } from './domain';

export const store = configureStore({
  reducer: RootSlice.reducer,
  devTools: true,
});

export async function loadStore() {
  const userSettings = await loadUserSettings();
  const initialState = RootSlice.getInitialState();

  const settings = JsonToState_WManager(userSettings.jsonSettings, initialState.settings);
  store.dispatch(RootActions.setSettings(settings));

  if (userSettings.theme) {
    loadThemeCSS(userSettings.theme);
    store.dispatch(RootActions.setTheme(userSettings.theme));
  }
}

export async function registerStoreEvents() {
  await listen<SeelenManagerState>('update-store-settings', (event) => {
    store.dispatch(RootActions.setSettings(event.payload));
  });

  await listen<Theme>('update-store-theme', (event) => {
    loadThemeCSS(event.payload);
    store.dispatch(RootActions.setTheme(event.payload));
  });

  await listen<{ hwnd: number; desktop_id: DesktopId }>('add-window', (event) => {
    store.dispatch(RootActions.addWindow(event.payload));
  });

  await listen<number>('remove-window', (event) => {
    store.dispatch(RootActions.removeWindow(event.payload));
  });

  await listen<void>('force-retiling', () => {
    store.dispatch(RootActions.forceUpdate());
  });

  await listen<DesktopId>('set-active-workspace', (event) => {
    store.dispatch(RootActions.setActiveWorkspace(event.payload));
  });

  await listen<number>('set-active-window', (event) => {
    store.dispatch(RootActions.setActiveWindow(event.payload));
    if (event.payload != 0) {
      store.dispatch(RootActions.setLastManagedActivated(event.payload));
    }
  });

  await listen<Reservation | null>('set-reservation', (event) => {
    store.dispatch(RootActions.setReservation(event.payload));
  });

  await listen<Sizing>('update-width', (event) => {
    store.dispatch(RootActions.updateSizing({ axis: 'x', sizing: event.payload }));
  });

  await listen<Sizing>('update-height', (event) => {
    store.dispatch(RootActions.updateSizing({ axis: 'y', sizing: event.payload }));
  });

  await listen<void>('reset-workspace-size', () => {
    store.dispatch(RootActions.resetSizing());
  });

  await listen<FocusAction>('focus', (event) => {
    store.dispatch(RootActions.focus(event.payload));
  });

  await listen<{ hwnd: HWND; desktop_id: DesktopId }>('move-window-to-workspace', (event) => {
    store.dispatch(RootActions.removeWindow(event.payload.hwnd));
    store.dispatch(RootActions.addWindow(event.payload));
  });
}