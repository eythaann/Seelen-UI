import { UserSettings } from '../../../../../shared.interfaces';
import { loadUserSettings } from '../../../../settings/modules/shared/store/storeApi';
import { loadThemeCSS } from '../../../../utils';
import { WindowManager } from '../../../../utils/schemas/WindowManager';
import { configureStore } from '@reduxjs/toolkit';
import { listen } from '@tauri-apps/api/event';

import { RootActions, RootSlice } from './app';

import { Reservation, Sizing } from '../../layout/domain';
import { AddWindowPayload, DesktopId, FocusAction } from './domain';

export const store = configureStore({
  reducer: RootSlice.reducer,
  devTools: true,
});

export async function loadStore() {
  const userSettings = await loadUserSettings();
  const settings = userSettings.jsonSettings.windowManager;
  store.dispatch(RootActions.setAvailableLayouts(userSettings.layouts));
  store.dispatch(RootActions.setSettings(settings));
  loadSettingsCSS(settings);
  if (userSettings.bgLayers) {
    loadThemeCSS(userSettings);
    store.dispatch(RootActions.setThemeLayers(userSettings.bgLayers));
  }
}

export async function registerStoreEvents() {
  await listen<UserSettings>('updated-settings', (event) => {
    const userSettings = event.payload;

    const settings = userSettings.jsonSettings.windowManager;
    loadSettingsCSS(settings);
    store.dispatch(RootActions.setAvailableLayouts(userSettings.layouts));
    store.dispatch(RootActions.setSettings(settings));
    if (userSettings.bgLayers) {
      loadThemeCSS(userSettings);
      store.dispatch(RootActions.setThemeLayers(userSettings.bgLayers));
    }
  });

  await listen<AddWindowPayload>('add-window', (event) => {
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

  await listen<AddWindowPayload>('move-window-to-workspace', (event) => {
    store.dispatch(RootActions.removeWindow(event.payload.hwnd));
    store.dispatch(RootActions.addWindow(event.payload));
  });
}

function loadSettingsCSS(settings: WindowManager) {
  const styles = document.documentElement.style;

  styles.setProperty('--config-padding', `${settings.workspacePadding}px`);
  styles.setProperty('--config-containers-gap', `${settings.workspaceGap}px`);

  styles.setProperty('--config-margin-top', `${settings.globalWorkAreaOffset.top}px`);
  styles.setProperty('--config-margin-left', `${settings.globalWorkAreaOffset.left}px`);
  styles.setProperty('--config-margin-right', `${settings.globalWorkAreaOffset.right}px`);
  styles.setProperty('--config-margin-bottom', `${settings.globalWorkAreaOffset.bottom}px`);

  styles.setProperty('--config-border-offset', `${settings.border.offset}px`);
  styles.setProperty('--config-border-width', `${settings.border.width}px`);
}
