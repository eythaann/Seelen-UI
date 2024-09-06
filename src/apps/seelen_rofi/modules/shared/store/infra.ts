import { StartThemingTool } from '../../../../shared/styles';
import { configureStore } from '@reduxjs/toolkit';
import { invoke } from '@tauri-apps/api/core';
import { InvokeHandler, UIColors } from 'seelen-core';

import { RootActions, RootSlice } from './app';

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

export async function initStore() {
  store.dispatch(RootActions.setApps(await invoke(InvokeHandler.GetLauncherApps)));

  await initUIColors();
  await StartThemingTool();
}