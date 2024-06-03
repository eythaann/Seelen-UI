import { UserSettings } from '../../../../../shared.interfaces';
import { loadUserSettings } from '../../../../settings/modules/shared/store/storeApi';
import { loadThemeCSS } from '../../../../utils';
import { FancyToolbar } from '../../../../utils/schemas/FancyToolbar';
import { configureStore } from '@reduxjs/toolkit';
import { listen as listenGlobal } from '@tauri-apps/api/event';
import { getCurrent } from '@tauri-apps/api/webviewWindow';

import { RootActions, RootSlice } from './app';

import { ActiveApp, PowerStatus } from './domain';

export const store = configureStore({
  reducer: RootSlice.reducer,
});

export async function registerStoreEvents() {
  const view = getCurrent();

  await view.listen<ActiveApp | null>('focus-changed', (e) => {
    store.dispatch(RootActions.setFocused(e.payload));
  });

  await listenGlobal<UserSettings>('updated-settings', (event) => {
    loadStore(event.payload);
  });

  await listenGlobal<PowerStatus>('power-status', (event) => {
    store.dispatch(RootActions.setPowerStatus(event.payload));
  });

  await listenGlobal<string[]>('workspaces-changed', (event) => {
    store.dispatch(RootActions.setWorkspaces(event.payload));
  });

  await listenGlobal<number>('active-workspace-changed', (event) => {
    store.dispatch(RootActions.setActiveWorkspace(event.payload));
  });

  await view.emitTo(view.label, 'store-events-ready');
}

export async function loadStore(_userSettings?: UserSettings) {
  const userSettings = _userSettings || (await loadUserSettings());
  const settings = userSettings.jsonSettings.fancyToolbar;

  loadSettingsCSS(settings);
  store.dispatch(RootActions.setSettings(settings));

  if (userSettings.bgLayers) {
    loadThemeCSS(userSettings);
    store.dispatch(RootActions.setThemeLayers(userSettings.bgLayers));
  }

  const placeholder =
    userSettings.placeholders.find(
      (placeholder) => placeholder.info.filename === settings.placeholder,
    ) || null;

  store.dispatch(RootActions.setPlaceholder(placeholder));
  store.dispatch(RootActions.setEnv(userSettings.env));
}

export function loadSettingsCSS(settings: FancyToolbar) {
  const styles = document.documentElement.style;

  styles.setProperty('--config-height', `${settings.height}px`);
}