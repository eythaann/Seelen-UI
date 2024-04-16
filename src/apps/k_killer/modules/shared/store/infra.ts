import { configureStore } from '@reduxjs/toolkit';
import { listen } from '@tauri-apps/api/event';

import { RootActions, RootSlice } from './app';

import { DesktopId } from './domain';

export const store = configureStore({
  reducer: RootSlice.reducer,
  devTools: true,
});

export type AppDispatch = typeof store.dispatch;
export type store = {
  dispatch: AppDispatch;
  getState: () => {};
};

export async function registerStoreEvents() {
  await listen<{ hwnd: number; desktop_id: DesktopId }>('add-window', (event) => {
    store.dispatch(RootActions.addWindow(event.payload));
  });

  await listen<number>('remove-window', (event) => {
    store.dispatch(RootActions.removeWindow(event.payload));
  });

  await listen<void>('force-retiling', () => {
    store.dispatch(RootActions.forceUpdate());
  });

  await listen<DesktopId>('set-active-workspace', (event) => {
    store.dispatch(RootActions.setActiveWorkspace(event.payload));
  });
}