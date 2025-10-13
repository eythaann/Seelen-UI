import { configureStore } from "@reduxjs/toolkit";
import { SeelenCommand, SeelenEvent, Settings, startThemingTool, subscribe } from "@seelen-ui/lib";
import type { FocusedApp, SeelenWegSettings } from "@seelen-ui/lib/types";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { debounce } from "lodash";

import { RootActions, RootSlice } from "./app.ts";

import i18n from "../../../i18n/index.ts";

export const store = configureStore({
  reducer: RootSlice.reducer,
  middleware(getDefaultMiddleware) {
    return getDefaultMiddleware({
      serializableCheck: false,
    });
  },
});

export async function registerStoreEvents() {
  const view = getCurrentWebviewWindow();

  const onFocusChanged = debounce((app: FocusedApp) => {
    store.dispatch(RootActions.setFocusedApp(app));
  }, 200);
  await view.listen<FocusedApp>(SeelenEvent.GlobalFocusChanged, (e) => {
    onFocusChanged(e.payload);
    if (e.payload.name != "Seelen UI") {
      onFocusChanged.flush();
    }
  });

  await subscribe(SeelenEvent.MediaSessions, (event) => {
    store.dispatch(RootActions.setMediaSessions(event.payload));
  });

  await subscribe(SeelenEvent.Notifications, (event) => {
    store.dispatch(RootActions.setNotifications(event.payload));
  });

  await Settings.onChange(loadSettingsToStore);

  await startThemingTool();
}

function loadSettingsCSS(settings: SeelenWegSettings) {
  const styles = document.documentElement.style;

  styles.setProperty("--config-margin", `${settings.margin}px`);
  styles.setProperty("--config-padding", `${settings.padding}px`);

  styles.setProperty("--config-item-size", `${settings.size}px`);
  styles.setProperty("--config-item-zoom-size", `${settings.zoomSize}px`);
  styles.setProperty(
    "--config-space-between-items",
    `${settings.spaceBetweenItems}px`,
  );
}

function loadSettingsToStore(settings: Settings) {
  i18n.changeLanguage(settings.inner.language || undefined);
  store.dispatch(RootActions.setDevTools(settings.inner.devTools));
  loadSettingsCSS(settings.seelenweg);
}

export async function loadStore() {
  loadSettingsToStore(await Settings.getAsync());
  store.dispatch(
    RootActions.setMediaSessions(await invoke(SeelenCommand.GetMediaSessions)),
  );
}
