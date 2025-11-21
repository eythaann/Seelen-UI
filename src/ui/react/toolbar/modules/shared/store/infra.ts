import { configureStore } from "@reduxjs/toolkit";
import {
  BluetoothDevices,
  DocumentsFolder,
  DownloadsFolder,
  LanguageList,
  MusicFolder,
  PicturesFolder,
  RecentFolder,
  SeelenEvent,
  Settings,
  startThemingTool,
  subscribe,
  UserDetails,
  VideosFolder,
} from "@seelen-ui/lib";
import type { FancyToolbarSettings } from "@seelen-ui/lib/types";
import { throttle } from "lodash";

import { lazySlice, RootActions, RootSlice } from "./app.ts";

import i18n from "../../../i18n/index.ts";

export const store = configureStore({
  reducer: RootSlice.reducer,
  middleware(getDefaultMiddleware) {
    return getDefaultMiddleware({
      serializableCheck: false,
    });
  },
});

lazySlice(store.dispatch);

export async function registerStoreEvents() {
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

  LanguageList.onChange((list) => store.dispatch(RootActions.setLanguages(list.asArray())));

  UserDetails.onChange((details) => store.dispatch(RootActions.setUser(details.user)));
  RecentFolder.onChange((details) => store.dispatch(RootActions.setUserRecentFolder(details.all())));
  DocumentsFolder.onChange((details) => store.dispatch(RootActions.setUserDocumentsFolder(details.all())));
  DownloadsFolder.onChange((details) => store.dispatch(RootActions.setUserDownloadsFolder(details.all())));
  PicturesFolder.onChange((details) => store.dispatch(RootActions.setUserPicturesFolder(details.all())));
  VideosFolder.onChange((details) => store.dispatch(RootActions.setUserVideosFolder(details.all())));
  MusicFolder.onChange((details) => store.dispatch(RootActions.setUserMusicFolder(details.all())));

  BluetoothDevices.onChange((devices) => store.dispatch(RootActions.setBluetoothDevices(devices.all())));
  BluetoothDevices.onDiscoveredDevicesChange((devices) =>
    store.dispatch(RootActions.setDiscoveredBluetoothDevices(devices.all()))
  );

  await startThemingTool();
}

function loadSettingsCSS(settings: FancyToolbarSettings) {
  const styles = document.documentElement.style;
  styles.setProperty("--config-height", `${settings.height}px`);
  styles.setProperty("--config-time-before-show", `${settings.delayToShow}ms`);
  styles.setProperty("--config-time-before-hide", `${settings.delayToHide}ms`);
}

function loadSettings(settings: Settings) {
  i18n.changeLanguage(settings.inner.language || undefined);
  loadSettingsCSS(settings.fancyToolbar);
}
