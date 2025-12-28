import { configureStore } from "@reduxjs/toolkit";
import { LauncherHistory, SeelenCommand, Settings, UIColors } from "@seelen-ui/lib";
import { invoke } from "@tauri-apps/api/core";

import { Actions, RootSlice } from "./app.ts";

import i18n from "../../../i18n/index.ts";

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
    store.dispatch(Actions.setColors(colors.inner));
  }
  loadColors(await UIColors.getAsync());
  await UIColors.onChange(loadColors);
}

export async function initStore() {
  const dispatch = store.dispatch;
  const settings = await Settings.getAsync();

  i18n.changeLanguage(settings.inner.language || undefined);

  dispatch(Actions.setSettings(settings.launcher));
  dispatch(Actions.setApps(await invoke(SeelenCommand.LauncherGetApps)));
  dispatch(Actions.setHistory((await LauncherHistory.getAsync()).inner));

  LauncherHistory.onChange((history) => dispatch(Actions.setHistory(history.inner)));
  Settings.onChange((settings) => {
    i18n.changeLanguage(settings.inner.language || undefined);
    dispatch(Actions.setSettings(settings.launcher));
  });

  await initUIColors();
}
