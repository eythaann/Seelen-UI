import { configureStore } from '@reduxjs/toolkit';
import { invoke } from '@tauri-apps/api/core';
import { LauncherHistory, SeelenCommand, Settings, UIColors } from 'seelen-core';

import { Actions, RootSlice } from './app';

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
    store.dispatch(Actions.setColors(colors));
  }
  loadColors(await UIColors.getAsync());
  await UIColors.onChange(loadColors);
}

export async function initStore() {
  const dispatch = store.dispatch;
  const settings = await Settings.getAsync();

  i18n.changeLanguage(settings.language);

  dispatch(Actions.setSettings(settings.launcher));
  dispatch(Actions.setApps(await invoke(SeelenCommand.LauncherGetApps)));
  dispatch(Actions.setHistory(await LauncherHistory.getAsync()));

  LauncherHistory.onChange((history) => dispatch(Actions.setHistory(history)));
  Settings.onChange((settings) => {
    i18n.changeLanguage(settings.language);
    dispatch(Actions.setSettings(settings.launcher));
  });

  await initUIColors();
  await StartThemingTool();
}
