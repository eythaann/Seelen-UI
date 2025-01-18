import { configureStore } from '@reduxjs/toolkit';
import {
  invoke,
  PlaceholderList,
  PluginList,
  SeelenCommand,
  SeelenEvent,
  Settings,
  UIColors,
} from '@seelen-ui/lib';
import { FancyToolbarSettings } from '@seelen-ui/lib/types';
import { listen as listenGlobal } from '@tauri-apps/api/event';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { debounce, throttle } from 'lodash';

import { RootActions, RootSlice } from './app';

import { WlanBssEntry } from '../../network/domain';
import { AppNotification } from '../../Notifications/domain';
import {
  Battery,
  MediaChannelTransportData,
  MediaDevice,
  NetworkAdapter,
  PowerStatus,
  TrayInfo,
  Workspace,
  WorkspaceId,
} from './domain';

import { FocusedApp } from '../../../../shared/interfaces/common';
import { StartThemingTool } from '../../../../shared/styles';
import i18n from '../../../i18n';

export const store = configureStore({
  reducer: RootSlice.reducer,
  middleware(getDefaultMiddleware) {
    return getDefaultMiddleware({
      serializableCheck: false,
    });
  },
});

async function initUIColors() {
  function loadColors(colors: UIColors) {
    store.dispatch(RootActions.setColors(colors.inner));
  }
  loadColors(await UIColors.getAsync());
  await UIColors.onChange(loadColors);
}

function setPlaceholder(list: PlaceholderList) {
  const state = store.getState();
  const placeholder = list
    .asArray()
    .find((placeholder) => placeholder.info.filename === state.settings.placeholder);
  store.dispatch(RootActions.setPlaceholder(placeholder || list.asArray()[0] || null));
}

export async function registerStoreEvents() {
  const view = getCurrentWebviewWindow();

  await view.listen<boolean>('set-auto-hide', (event) => {
    store.dispatch(RootActions.setIsOverlaped(event.payload));
  });

  const onFocusChanged = debounce((app: FocusedApp) => {
    store.dispatch(RootActions.setFocused(app));
  }, 200);
  await listenGlobal<FocusedApp>(SeelenEvent.GlobalFocusChanged, (e) => {
    onFocusChanged(e.payload);
    if (e.payload.name != 'Seelen UI') {
      onFocusChanged.flush();
    }
  });

  await Settings.onChange(loadStore);

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

  await PlaceholderList.onChange(setPlaceholder);

  store.dispatch(RootActions.setPlugins((await PluginList.getAsync()).forCurrentWidget()));
  await PluginList.onChange((list) => {
    store.dispatch(RootActions.setPlugins(list.forCurrentWidget()));
  });

  await initUIColors();
  await StartThemingTool();
  await view.emitTo(view.label, 'store-events-ready');
}

export async function loadStore() {
  const settings = await Settings.getAsync();

  i18n.changeLanguage(settings.inner.language || undefined);

  loadSettingsCSS(settings.fancyToolbar);
  store.dispatch(RootActions.setSettings(settings.fancyToolbar));
  store.dispatch(RootActions.setDateFormat(settings.inner.dateFormat));
  store.dispatch(
    RootActions.setEnv((await invoke(SeelenCommand.GetUserEnvs)) as Record<string, string>),
  );

  PlaceholderList.getAsync().then(setPlaceholder);
}

export function loadSettingsCSS(settings: FancyToolbarSettings) {
  const styles = document.documentElement.style;

  styles.setProperty('--config-height', `${settings.height}px`);
  styles.setProperty('--config-time-before-show', `${settings.delayToShow}ms`);
  styles.setProperty('--config-time-before-hide', `${settings.delayToHide}ms`);
}
