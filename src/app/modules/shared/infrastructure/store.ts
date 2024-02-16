import { configureStore } from '@reduxjs/toolkit';

import { mainReducer } from '../app/reducer';

import { GlobalState } from '../domain/state';

export const store = configureStore({
  reducer: mainReducer,
});

export type AppDispatch = typeof store.dispatch;
export type store = {
  dispatch: AppDispatch;
  getState: () => GlobalState;
};