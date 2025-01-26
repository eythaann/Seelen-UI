import { configureStore } from '@reduxjs/toolkit';
import {
  ApplicationHistory,
  DocumentsFolder,
  DownloadsFolder,
  invoke,
  MusicFolder,
  PicturesFolder,
  PluginList,
  RecentFolder,
  SeelenCommand,
  SeelenEvent,
  Settings,
  UIColors,
  UserDetails,
  VideosFolder,
} from '@seelen-ui/lib';
import { FancyToolbarSettings, Placeholder } from '@seelen-ui/lib/types';
import { listen as listenGlobal } from '@tauri-apps/api/event';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { throttle } from 'lodash';

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

export async function registerStoreEvents() {
  const view = getCurrentWebviewWindow();

  await view.listen<boolean>('set-auto-hide', (event) => {
    store.dispatch(RootActions.setIsOverlaped(event.payload));
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

  await listenGlobal<Placeholder>(SeelenEvent.StateToolbarItemsChanged, (event) => {
    store.dispatch(RootActions.setPlaceholder(event.payload));
  });

  store.dispatch(RootActions.setPlugins((await PluginList.getAsync()).forCurrentWidget()));
  await PluginList.onChange((list) => {
    store.dispatch(RootActions.setPlugins(list.forCurrentWidget()));
  });

  ApplicationHistory.onFocusChanged((app) => store.dispatch(RootActions.setFocused(app.payload)));
  ApplicationHistory.onChange((history) => store.dispatch(RootActions.setHistory(history.all())));
  ApplicationHistory.onCurrentMonitorHistoryChanged((history) => store.dispatch(RootActions.setHistoryOnMonitor(history.all())));

  UserDetails.onChange((details) => store.dispatch(RootActions.setUser(details.user)));
  RecentFolder.onChange((details) => store.dispatch(RootActions.setUserRecentFolder(details.all())));
  DocumentsFolder.onChange((details) => store.dispatch(RootActions.setUserDocumentsFolder(details.all())));
  DownloadsFolder.onChange((details) => store.dispatch(RootActions.setUserDownloadsFolder(details.all())));
  PicturesFolder.onChange((details) => store.dispatch(RootActions.setUserPicturesFolder(details.all())));
  VideosFolder.onChange((details) => store.dispatch(RootActions.setUserVideosFolder(details.all())));
  MusicFolder.onChange((details) => store.dispatch(RootActions.setUserMusicFolder(details.all())));

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

  store.dispatch(RootActions.setHistory((await ApplicationHistory.getAsync()).all()));
  store.dispatch(RootActions.setHistoryOnMonitor((await ApplicationHistory.getCurrentMonitorHistoryAsync()).all()));

  store.dispatch(async () => {
    store.dispatch(RootActions.setUser((await UserDetails.getAsync()).user));
    store.dispatch(RootActions.setUserRecentFolder((await RecentFolder.getAsync()).all()));
    store.dispatch(RootActions.setUserDocumentsFolder((await DocumentsFolder.getAsync()).all()));
    store.dispatch(RootActions.setUserDownloadsFolder((await DownloadsFolder.getAsync()).all()));
    store.dispatch(RootActions.setUserPicturesFolder((await PicturesFolder.getAsync()).all()));
    store.dispatch(RootActions.setUserVideosFolder((await VideosFolder.getAsync()).all()));
    store.dispatch(RootActions.setUserMusicFolder((await MusicFolder.getAsync()).all()));
  });

  let placeholder = await invoke(SeelenCommand.StateGetToolbarItems) as Placeholder;
  store.dispatch(RootActions.setPlaceholder(placeholder));
}

export function loadSettingsCSS(settings: FancyToolbarSettings) {
  const styles = document.documentElement.style;

  styles.setProperty('--config-height', `${settings.height}px`);
  styles.setProperty('--config-time-before-show', `${settings.delayToShow}ms`);
  styles.setProperty('--config-time-before-hide', `${settings.delayToHide}ms`);
}
