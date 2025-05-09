import { configureStore } from '@reduxjs/toolkit';
import {
  BluetoothDevices,
  BluetoothRadio,
  Color,
  DocumentsFolder,
  DownloadsFolder,
  IColor,
  LanguageList,
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
import { invoke } from '@tauri-apps/api/core';
import { listen as listenGlobal } from '@tauri-apps/api/event';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { debounce, throttle } from 'lodash';

import { lazySlice, RootActions, RootSlice } from './app';
import { FocusedApp } from 'src/apps/shared/interfaces/common';

import { WlanBssEntry } from '../../network/domain';
import { AppNotification } from '../../Notifications/domain';
import {
  Battery,
  MediaChannelTransportData,
  MediaDevice,
  NetworkAdapter,
  PowerPlan,
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

lazySlice(store.dispatch);

async function initUIColors() {
  function loadColors(colors: UIColors) {
    store.dispatch(RootActions.setColors(colors.inner));
  }
  loadColors(await UIColors.getAsync());
  await UIColors.onChange(loadColors);
}

const removeFocusedColorCssVars = () => {
  document.documentElement.style.removeProperty('--color-focused-app-background');
  document.documentElement.style.removeProperty('--color-focused-app-foreground');
};

async function initFocusedColorSystem() {
  let optimisticFocused: FocusedApp | null = null;
  const setFocused = debounce((app: FocusedApp) => {
    store.dispatch(RootActions.setFocused(app));
  }, 200);

  const updateFocusedColor = async () => {
    const { settings } = store.getState();
    if (!settings.dynamicColor || optimisticFocused?.isSeelenOverlay) {
      return;
    }

    let color = new Color(await invoke<IColor>(SeelenCommand.SystemGetForegroundWindowColor));
    if (color.inner.a === 0) {
      removeFocusedColorCssVars();
      return;
    }

    // like this is an async operation sometimes this is not skiped on first condition
    if (optimisticFocused?.isSeelenOverlay) {
      return;
    }

    let luminance = color.calcLuminance();
    document.documentElement.style.setProperty('--color-focused-app-background', color.asHex());
    document.documentElement.style.setProperty(
      '--color-focused-app-foreground',
      luminance / 255 > 0.5 ? 'var(--color-persist-gray-900)' : 'var(--color-persist-gray-100)',
    );
  };

  await listenGlobal('hidden::remove-focused-color', removeFocusedColorCssVars);

  window.setInterval(updateFocusedColor, 350);
  await listenGlobal<FocusedApp>(SeelenEvent.GlobalFocusChanged, (e) => {
    const app = e.payload;
    optimisticFocused = app;
    setFocused(app);
    if (!app.isSeelenOverlay) {
      setFocused.flush();
    }
    updateFocusedColor();
  });
}

export async function registerStoreEvents() {
  const view = getCurrentWebviewWindow();

  await view.listen<boolean>('set-auto-hide', (event) => {
    store.dispatch(RootActions.setIsOverlaped(event.payload));
  });

  Settings.getAsync().then(loadSettings);
  Settings.onChange(loadSettings);

  await listenGlobal<PowerStatus>(SeelenEvent.PowerStatus, (event) => {
    store.dispatch(RootActions.setPowerStatus(event.payload));
  });

  await listenGlobal<PowerPlan>(SeelenEvent.PowerPlan, (event) => {
    store.dispatch(RootActions.setPowerPlan(event.payload));
  });

  await listenGlobal<Battery[]>(SeelenEvent.BatteriesStatus, (event) => {
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

  await listenGlobal<AppNotification[]>(SeelenEvent.Notifications, (event) => {
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

  LanguageList.onChange((list) => store.dispatch(RootActions.setLanguages(list.asArray())));

  UserDetails.onChange((details) => store.dispatch(RootActions.setUser(details.user)));
  RecentFolder.onChange((details) =>
    store.dispatch(RootActions.setUserRecentFolder(details.all())),
  );
  DocumentsFolder.onChange((details) =>
    store.dispatch(RootActions.setUserDocumentsFolder(details.all())),
  );
  DownloadsFolder.onChange((details) =>
    store.dispatch(RootActions.setUserDownloadsFolder(details.all())),
  );
  PicturesFolder.onChange((details) =>
    store.dispatch(RootActions.setUserPicturesFolder(details.all())),
  );
  VideosFolder.onChange((details) =>
    store.dispatch(RootActions.setUserVideosFolder(details.all())),
  );
  MusicFolder.onChange((details) => store.dispatch(RootActions.setUserMusicFolder(details.all())));

  BluetoothDevices.onChange((devices) =>
    store.dispatch(RootActions.setBluetoothDevices(devices.all())),
  );
  BluetoothDevices.onDiscoveredDevicesChange((devices) =>
    store.dispatch(RootActions.setDiscoveredBluetoothDevices(devices.all())),
  );
  BluetoothRadio.onChange((radio) =>
    store.dispatch(RootActions.setBluetoothRadioState(radio.state)),
  );

  await initFocusedColorSystem();

  await initUIColors();
  await StartThemingTool();
  await view.emitTo(view.label, 'store-events-ready');
}

function loadSettingsCSS(settings: FancyToolbarSettings) {
  const styles = document.documentElement.style;
  styles.setProperty('--config-height', `${settings.height}px`);
  styles.setProperty('--config-time-before-show', `${settings.delayToShow}ms`);
  styles.setProperty('--config-time-before-hide', `${settings.delayToHide}ms`);
}

async function loadSettings(settings: Settings) {
  i18n.changeLanguage(settings.inner.language || undefined);

  store.dispatch(RootActions.setSettings(settings.fancyToolbar));
  store.dispatch(RootActions.setDateFormat(settings.inner.dateFormat));

  loadSettingsCSS(settings.fancyToolbar);
  if (!settings.fancyToolbar.dynamicColor) {
    removeFocusedColorCssVars();
  }
}
