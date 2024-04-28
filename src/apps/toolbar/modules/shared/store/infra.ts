import { configureStore } from '@reduxjs/toolkit';
import { getCurrent } from '@tauri-apps/api/webviewWindow';

import { RootActions, RootSlice } from './app';

import { ActiveApp } from './domain';

export const store = configureStore({
  reducer: RootSlice.reducer,
});

export async function registerStoreEvents() {
  const webview = getCurrent();

  await webview.listen<ActiveApp | null>('focus-changed', (e) => {
    store.dispatch(RootActions.setFocused(e.payload));
  });
}