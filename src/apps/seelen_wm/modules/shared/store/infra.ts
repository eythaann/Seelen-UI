import { UserSettings } from '../../../../../shared.interfaces';
import { loadThemeCSS } from '../../../../utils';
import { configureStore } from '@reduxjs/toolkit';
import { listen } from '@tauri-apps/api/event';
import { defaultsDeep } from 'lodash';

import { loadUserSettings } from '../../../../settings/modules/shared/infrastructure/storeApi';

import { VariableConvention } from '../../../../settings/modules/shared/app/utils';
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

  let settings = VariableConvention.fromSnakeToCamel(userSettings.jsonSettings.seelen_wm || {}) as any;
  settings = defaultsDeep(settings, initialState.settings);

  store.dispatch(RootActions.setSettings(settings));
  loadSettingsCSS(settings);
  if (userSettings.theme) {
    loadThemeCSS(userSettings.theme);
    store.dispatch(RootActions.setTheme(userSettings.theme));
  }
}

export async function registerStoreEvents() {
  await listen<UserSettings>('updated-settings', (event) => {
    const currentState = store.getState();
    const userSettings = event.payload;

    let settings = VariableConvention.fromSnakeToCamel(userSettings.jsonSettings.seelen_wm || {}) as any;
    settings = defaultsDeep(settings, currentState.settings);

    loadSettingsCSS(settings);
    store.dispatch(RootActions.setSettings(settings));
    if (userSettings.theme) {
      loadThemeCSS(userSettings.theme, currentState.theme);
      store.dispatch(RootActions.setTheme(userSettings.theme));
    }
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

function loadSettingsCSS(settings: SeelenManagerState) {
  const styles = document.documentElement.style;

  styles.setProperty('--config-padding', `${settings.workspacePadding}px`);
  styles.setProperty('--config-containers-gap', `${settings.containerPadding}px`);

  styles.setProperty('--config-margin-top', `${settings.globalWorkAreaOffset.top}px`);
  styles.setProperty('--config-margin-left', `${settings.globalWorkAreaOffset.left}px`);
  styles.setProperty('--config-margin-right', `${settings.globalWorkAreaOffset.right}px`);
  styles.setProperty('--config-margin-bottom', `${settings.globalWorkAreaOffset.bottom}px`);

  styles.setProperty('--config-border-offset', `${settings.border.offset}px`);
  styles.setProperty('--config-border-width', `${settings.border.width}px`);
}