import { UIColors } from '../../../lib/src/system_state';
import { StartThemingTool } from '../shared/styles';
import { RootActions, RootSlice } from './reducer';
import { configureStore } from '@reduxjs/toolkit';

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
  await initUIColors();
  await StartThemingTool();
}