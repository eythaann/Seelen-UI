import { configureStore } from '@reduxjs/toolkit';
import { Settings } from 'seelen-core';

import { Actions, RootSlice } from './app';

export const store = configureStore({
  reducer: RootSlice.reducer,
  middleware(getDefaultMiddleware) {
    return getDefaultMiddleware({
      serializableCheck: false,
    });
  },
});

export async function initStore() {
  const settings = await Settings.getAsync();

  store.dispatch(Actions.setSettings(settings.wall));
  Settings.onChange((settings) => {
    store.dispatch(Actions.setSettings(settings.wall));
  });
}
