import { configureStore } from '@reduxjs/toolkit';

import { RootSlice } from './app';

export const store = configureStore({
  reducer: RootSlice.reducer,
});

export type AppDispatch = typeof store.dispatch;
export type store = {
  dispatch: AppDispatch;
  getState: () => {};
};