import { configureStore } from '@reduxjs/toolkit';
import {
  BluetoothDevices,
  Color,
  DocumentsFolder,
  DownloadsFolder,
  LanguageList,
  MusicFolder,
  PicturesFolder,
  PluginList,
  RecentFolder,
  SeelenCommand,
  SeelenEvent,
  Settings,
  subscribe,
  UIColors,
  UserDetails,
  VideosFolder,
} from '@seelen-ui/lib';
import { FancyToolbarSettings, FocusedApp } from '@seelen-ui/lib/types';
import { invoke } from '@tauri-apps/api/core';
import { listen as listenGlobal } from '@tauri-apps/api/event';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { debounce, throttle } from 'lodash';

import { lazySlice, RootActions, RootSlice } from './app';

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
    if (!optimisticFocused || !settings.dynamicColor) {
      return;
    }

    let color = new Color(await invoke(SeelenCommand.SystemGetForegroundWindowColor));
    if (color.inner.a === 0) {
      removeFocusedColorCssVars();
      return;
    }

    const luminance = color.calcLuminance();
    const background = color.asHex();
    const foreground =
      luminance / 255 > 0.5 ? 'var(--color-persist-gray-900)' : 'var(--color-persist-gray-100)';

    document.documentElement.style.setProperty('--color-focused-app-background', background);
    document.documentElement.style.setProperty('--color-focused-app-foreground', foreground);

    store.dispatch(
      RootActions.addWindowColor([optimisticFocused.hwnd, { background, foreground }]),
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

  await subscribe(SeelenEvent.PowerStatus, (event) => {
    store.dispatch(RootActions.setPowerStatus(event.payload));
  });

  await subscribe(SeelenEvent.PowerMode, (event) => {
    store.dispatch(RootActions.setPowerPlan(event.payload));
  });

  await subscribe(SeelenEvent.BatteriesStatus, (event) => {
    store.dispatch(RootActions.setBatteries(event.payload));
  });

  await subscribe(SeelenEvent.WorkspacesChanged, (event) => {
    store.dispatch(RootActions.setWorkspaces(event.payload));
  });

  await subscribe(SeelenEvent.ActiveWorkspaceChanged, (event) => {
    store.dispatch(RootActions.setActiveWorkspace(event.payload));
  });

  await subscribe(SeelenEvent.TrayInfo, (event) => {
    store.dispatch(RootActions.setSystemTray(event.payload));
  });

  await subscribe(SeelenEvent.MediaSessions, (event) => {
    store.dispatch(RootActions.setMediaSessions(event.payload));
  });

  await subscribe(
    SeelenEvent.MediaOutputs,
    throttle((event) => {
      store.dispatch(RootActions.setMediaOutputs(event.payload));
    }, 20),
  );

  await subscribe(SeelenEvent.MediaInputs, (event) => {
    store.dispatch(RootActions.setMediaInputs(event.payload));
  });

  await subscribe(SeelenEvent.Notifications, (event) => {
    store.dispatch(
      RootActions.setNotifications(event.payload.sort((a, b) => Number(b.date - a.date))),
    );
  });

  await subscribe(SeelenEvent.NetworkAdapters, (event) => {
    store.dispatch(RootActions.setNetworkAdapters(event.payload));
  });

  await subscribe(SeelenEvent.NetworkDefaultLocalIp, (event) => {
    store.dispatch(RootActions.setNetworkLocalIp(event.payload));
  });

  await subscribe(SeelenEvent.NetworkInternetConnection, (event) => {
    store.dispatch(RootActions.setOnline(event.payload));
  });

  await subscribe(SeelenEvent.NetworkWlanScanned, (event) => {
    store.dispatch(RootActions.setWlanBssEntries(event.payload));
  });

  await subscribe(SeelenEvent.StateToolbarItemsChanged, (event) => {
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
