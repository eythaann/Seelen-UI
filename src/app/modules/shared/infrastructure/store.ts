import { configureStore } from '@reduxjs/toolkit';

import { GlobalState } from '../domain/state';

export const store = configureStore({
  reducer: {},
});

export type AppDispatch = typeof store.dispatch;
export type store = {
  dispatch: AppDispatch;
  getState: () => GlobalState;
};