import { UIColors } from '../../../lib/src/system_state';
import { StartThemingTool } from '../shared/styles';
import { RootActions, RootSlice } from './reducer';
import { configureStore } from '@reduxjs/toolkit';

export const store = configureStore({
  reducer: RootSlice.reducer,
});

function loadColors(colors: UIColors) {
  UIColors.setAssCssVariables(colors);
  store.dispatch(RootActions.setColors(colors));
}

export async function initStore() {
  loadColors(await UIColors.getAsync());
  await UIColors.onChange(loadColors);

  await StartThemingTool();
}