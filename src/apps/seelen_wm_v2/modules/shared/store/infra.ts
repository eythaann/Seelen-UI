import { configureStore } from '@reduxjs/toolkit';
import { listen } from '@tauri-apps/api/event';
import { SeelenEvent, Settings, UIColors, WmNode } from 'seelen-core';

import { Actions, RootSlice } from './app';

import { StartThemingTool } from '../../../../shared/styles';

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
    store.dispatch(Actions.setColors(colors));
  }
  loadColors(await UIColors.getAsync());
  await UIColors.onChange(loadColors);
}

export async function loadStore() {
  await loadUIColors();

  setSettings(await Settings.getAsync());
  await Settings.onChange(setSettings);

  await listen<WmNode | null>(SeelenEvent.WMSetLayout, (e) => {
    store.dispatch(Actions.setLayout(e.payload));
  });

  await listen<void>(SeelenEvent.WMForceRetiling, () => {
    store.dispatch(Actions.forceUpdate());
  });

  await listen<boolean>(SeelenEvent.WMSetOverlayVisibility, ({ payload }) => {
    store.dispatch(Actions.setOverlayVisible(payload));
    document.body.style.opacity = payload ? '1' : '0';
  });

  await listen<number>(SeelenEvent.WMSetActiveWindow, ({ payload }) => {
    store.dispatch(Actions.setActiveWindow(payload));
  });

  await StartThemingTool();
}
