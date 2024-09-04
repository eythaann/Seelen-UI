import { UserSettings } from '../../../../../shared.interfaces';
import { UserSettingsLoader } from '../../../../settings/modules/shared/store/storeApi';
import { FileChange, GlobalEvent } from '../../../../shared/events';
import { FocusedApp } from '../../../../shared/interfaces/common';
import { StartThemingTool } from '../../../../shared/styles';
import i18n from '../../../i18n';
import { configureStore } from '@reduxjs/toolkit';
import { listen as listenGlobal } from '@tauri-apps/api/event';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { debounce, throttle } from 'lodash';
import { UIColors } from 'seelen-core';
import { FancyToolbarSettings } from 'seelen-core';

import { IsSavingCustom } from '../../main/application';
import { RootActions, RootSlice } from './app';

import { WlanBssEntry } from '../../network/domain';
import {
  AppNotification,
  Battery,
  MediaChannelTransportData,
  MediaDevice,
  NetworkAdapter,
  PowerStatus,
  TrayInfo,
  Workspace,
  WorkspaceId,
} from './domain';

export const store = configureStore({
  reducer: RootSlice.reducer,
});

async function initUIColors() {
  function loadColors(colors: UIColors) {
    UIColors.setAssCssVariables(colors);
    store.dispatch(RootActions.setColors(colors));
  }
  loadColors(await UIColors.getAsync());
  await UIColors.onChange(loadColors);
}

export async function registerStoreEvents() {
  const view = getCurrentWebviewWindow();

  await view.listen<boolean>('set-auto-hide', (event) => {
    store.dispatch(RootActions.setIsOverlaped(event.payload));
  });

  const onFocusChanged = debounce((app: FocusedApp) => {
    store.dispatch(RootActions.setFocused(app));
  }, 200);
  await listenGlobal<FocusedApp>(GlobalEvent.FocusChanged, (e) => {
    onFocusChanged(e.payload);
    if (e.payload.name != 'Seelen UI') {
      onFocusChanged.flush();
    }
  });

  await listenGlobal<any>(FileChange.Settings, async (_event) => {
    await loadStore();
  });

  await listenGlobal<PowerStatus>('power-status', (event) => {
    store.dispatch(RootActions.setPowerStatus(event.payload));
  });

  await listenGlobal<Battery[]>('batteries-status', (event) => {
    store.dispatch(RootActions.setBatteries(event.payload));
  });

  await listenGlobal<Workspace[]>('workspaces-changed', (event) => {
    store.dispatch(RootActions.setWorkspaces(event.payload));
  });

  await listenGlobal<WorkspaceId>('active-workspace-changed', (event) => {
    store.dispatch(RootActions.setActiveWorkspace(event.payload));
  });

  await listenGlobal<TrayInfo[]>('tray-info', (event) => {
    store.dispatch(RootActions.setSystemTray(event.payload));
  });

  await listenGlobal<MediaChannelTransportData[]>('media-sessions', (event) => {
    console.log(event.payload);
    store.dispatch(RootActions.setMediaSessions(event.payload));
  });

  await listenGlobal<MediaDevice[]>(
    'media-outputs',
    throttle((event) => {
      store.dispatch(RootActions.setMediaOutputs(event.payload));
    }, 20),
  );

  await listenGlobal<MediaDevice[]>('media-inputs', (event) => {
    store.dispatch(RootActions.setMediaInputs(event.payload));
  });

  await listenGlobal<AppNotification[]>('notifications', (event) => {
    store.dispatch(RootActions.setNotifications(event.payload.sort((a, b) => b.date - a.date)));
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

  await listenGlobal(FileChange.Placeholders, async () => {
    if (IsSavingCustom.current) {
      IsSavingCustom.current = false;
      return;
    }
    const userSettings = await new UserSettingsLoader().withPlaceholders().load();
    setPlaceholder(userSettings);
  });

  await initUIColors();
  await StartThemingTool();
  await view.emitTo(view.label, 'store-events-ready');
}

function setPlaceholder(userSettings: UserSettings) {
  const settings = userSettings.jsonSettings.fancyToolbar;
  const placeholder = settings.placeholder
    ? userSettings.placeholders.find(
      (placeholder) => placeholder.info.filename === settings.placeholder,
    )
    : null;
  store.dispatch(RootActions.setPlaceholder(placeholder || userSettings.placeholders[0] || null));
}

export async function loadStore() {
  const userSettings = await new UserSettingsLoader().withPlaceholders().load();
  const settings = userSettings.jsonSettings.fancyToolbar;
  i18n.changeLanguage(userSettings.jsonSettings.language);

  loadSettingsCSS(settings);
  store.dispatch(RootActions.setSettings(settings));

  setPlaceholder(userSettings);

  store.dispatch(RootActions.setEnv(userSettings.env));
}

export function loadSettingsCSS(settings: FancyToolbarSettings) {
  const styles = document.documentElement.style;

  styles.setProperty('--config-height', `${settings.height}px`);
}
