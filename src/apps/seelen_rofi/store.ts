import { StartThemingTool } from '../shared/styles';
import { RootSlice } from './reducer';
import { configureStore } from '@reduxjs/toolkit';

export const store = configureStore({
  reducer: RootSlice.reducer,
});

export async function initStore() {
  await StartThemingTool(store.dispatch);
}