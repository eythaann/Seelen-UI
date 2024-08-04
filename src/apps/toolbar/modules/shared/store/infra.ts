import { UserSettings } from '../../../../../shared.interfaces';
import { UserSettingsLoader } from '../../../../settings/modules/shared/store/storeApi';
import { loadThemeCSS, setAccentColorAsCssVar } from '../../../../shared';
import { FancyToolbar } from '../../../../shared/schemas/FancyToolbar';
import i18n from '../../../i18n';
import { configureStore } from '@reduxjs/toolkit';
import { invoke } from '@tauri-apps/api/core';
import { listen as listenGlobal } from '@tauri-apps/api/event';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';

import { RootActions, RootSlice } from './app';

import { WlanBssEntry } from '../../network/domain';
import {
  ActiveApp,
  AppNotification,
  Battery,
  MediaChannelTransportData,
  MediaDevice,
  NetworkAdapter,
  PowerStatus,
  TrayInfo,
} from './domain';

export const store = configureStore({
  reducer: RootSlice.reducer,
});

export async function registerStoreEvents() {
  const view = getCurrentWebviewWindow();

  await view.listen<ActiveApp | null>('focus-changed', (e) => {
    store.dispatch(RootActions.setFocused(e.payload));
  });

  await listenGlobal<UserSettings>('updated-settings', (event) => {
    loadStore(event.payload);
  });

  await listenGlobal<PowerStatus>('power-status', (event) => {
    store.dispatch(RootActions.setPowerStatus(event.payload));
  });

  await listenGlobal<Battery[]>('batteries-status', (event) => {
    store.dispatch(RootActions.setBatteries(event.payload));
  });

  await listenGlobal<string[]>('workspaces-changed', (event) => {
    store.dispatch(RootActions.setWorkspaces(event.payload));
  });

  await listenGlobal<number>('active-workspace-changed', (event) => {
    store.dispatch(RootActions.setActiveWorkspace(event.payload));
  });

  await listenGlobal<TrayInfo[]>('tray-info', (event) => {
    store.dispatch(RootActions.setSystemTray(event.payload));
  });

  await listenGlobal<MediaChannelTransportData[]>('media-sessions', (event) => {
    store.dispatch(RootActions.setMediaSessions(event.payload));
  });

  await listenGlobal<MediaDevice[]>('media-outputs', (event) => {
    store.dispatch(RootActions.setMediaOutputs(event.payload));
  });

  await listenGlobal<MediaDevice[]>('media-inputs', (event) => {
    store.dispatch(RootActions.setMediaInputs(event.payload));
  });

  await listenGlobal<AppNotification[]>('notifications', (event) => {
    store.dispatch(RootActions.setNotifications(event.payload));
  });

  await listenGlobal<NetworkAdapter[]>('network-adapters', (event) => {
    store.dispatch(RootActions.setNetworkAdapters(event.payload));
  });

  await listenGlobal<string | null>('network-default-local-ip', (event) => {
    store.dispatch(RootActions.setNetworkLocalIp(event.payload));
  });

  await listenGlobal<boolean>('network-internet-connection', (event) => {
    store.dispatch(RootActions.setOnline(event.payload));
  });

  await listenGlobal<WlanBssEntry[]>('wlan-scanned', (event) => {
    store.dispatch(RootActions.setWlanBssEntries(event.payload));
  });

  await listenGlobal<string>('theme-accent-color', (event) => {
    store.dispatch(RootActions.setAccentColor(event.payload));
  });

  await view.emitTo(view.label, 'store-events-ready');
}

export async function loadStore(_userSettings?: UserSettings) {
  const userSettings = _userSettings || (await new UserSettingsLoader().withPlaceholders().load());
  const settings = userSettings.jsonSettings.fancyToolbar;
  i18n.changeLanguage(userSettings.jsonSettings.language);

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

  invoke<string>('get_accent_color').then((color) => {
    setAccentColorAsCssVar(color);
    store.dispatch(RootActions.setAccentColor(color));
  });
}

export function loadSettingsCSS(settings: FancyToolbar) {
  const styles = document.documentElement.style;

  styles.setProperty('--config-height', `${settings.height}px`);
}
