import { configureStore } from '@reduxjs/toolkit';
import { Settings, startThemingTool, UIColors } from '@seelen-ui/lib';

import { Actions, RootSlice } from './app';

export const store = configureStore({
  reducer: RootSlice.reducer,
  middleware(getDefaultMiddleware) {
    return getDefaultMiddleware({
      serializableCheck: false,
    });
  },
});

function setSettings(_settings: Settings) {
  let settings = _settings.windowManager;
  store.dispatch(Actions.setSettings(settings));

  const styles = document.documentElement.style;

  styles.setProperty('--config-padding', `${settings.workspacePadding}px`);
  styles.setProperty('--config-containers-gap', `${settings.workspaceGap}px`);

  styles.setProperty('--config-margin-top', `${settings.workspaceMargin.top}px`);
  styles.setProperty('--config-margin-left', `${settings.workspaceMargin.left}px`);
  styles.setProperty('--config-margin-right', `${settings.workspaceMargin.right}px`);
  styles.setProperty('--config-margin-bottom', `${settings.workspaceMargin.bottom}px`);

  styles.setProperty('--config-border-offset', `${settings.border.offset}px`);
  styles.setProperty('--config-border-width', `${settings.border.width}px`);
}

async function loadUIColors() {
  function loadColors(colors: UIColors) {
    store.dispatch(Actions.setColors(colors.inner));
  }
  loadColors(await UIColors.getAsync());
  await UIColors.onChange(loadColors);
}

export async function loadStore() {
  await loadUIColors();

  setSettings(await Settings.getAsync());
  await Settings.onChange(setSettings);

  await startThemingTool();
}
