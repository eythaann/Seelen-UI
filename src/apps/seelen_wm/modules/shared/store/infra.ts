import { configureStore } from '@reduxjs/toolkit';
import { listen as listenGlobal } from '@tauri-apps/api/event';
import { SeelenEvent, UIColors, WindowManagerSettings } from 'seelen-core';

import { RootActions, RootSlice } from './app';

import { Reservation, Sizing } from '../../layout/domain';
import { AddWindowPayload, DesktopId, FocusAction } from './domain';

import { UserSettingsLoader } from '../../../../settings/modules/shared/store/storeApi';
import { StartThemingTool } from '../../../../shared/styles';

export const store = configureStore({
  reducer: RootSlice.reducer,
  middleware(getDefaultMiddleware) {
    return getDefaultMiddleware({
      serializableCheck: false,
    });
  },
});

export async function loadStore() {
  const userSettings = await new UserSettingsLoader().withLayouts().load();
  const settings = userSettings.jsonSettings.windowManager;
  store.dispatch(RootActions.setAvailableLayouts(userSettings.layouts));
  store.dispatch(RootActions.setSettings(settings));
  loadSettingsCSS(settings);
}

async function loadUIColors() {
  function loadColors(colors: UIColors) {
    store.dispatch(RootActions.setColors(colors));
  }
  loadColors(await UIColors.getAsync());
  await UIColors.onChange(loadColors);
}

export async function registerStoreEvents() {
  await loadUIColors();

  await listenGlobal<any>(SeelenEvent.StateSettingsChanged, async () => {
    await loadStore();
  });

  await listenGlobal<AddWindowPayload>('add-window', (event) => {
    store.dispatch(RootActions.addWindow(event.payload));
  });

  await listenGlobal<number>('remove-window', (event) => {
    store.dispatch(RootActions.removeWindow(event.payload));
  });

  await listenGlobal<void>('force-retiling', () => {
    store.dispatch(RootActions.forceUpdate());
  });

  await listenGlobal<DesktopId>('set-active-workspace', (event) => {
    store.dispatch(RootActions.setActiveWorkspace(event.payload));
  });

  await listenGlobal<number>('set-active-window', (event) => {
    store.dispatch(RootActions.setActiveWindow(event.payload));
    if (event.payload != 0) {
      store.dispatch(RootActions.setLastManagedActivated(event.payload));
    }
  });

  await listenGlobal<Reservation | null>('set-reservation', (event) => {
    store.dispatch(RootActions.setReservation(event.payload));
  });

  await listenGlobal<Sizing>('update-width', (event) => {
    store.dispatch(RootActions.updateSizing({ axis: 'x', sizing: event.payload }));
  });

  await listenGlobal<Sizing>('update-height', (event) => {
    store.dispatch(RootActions.updateSizing({ axis: 'y', sizing: event.payload }));
  });

  await listenGlobal<void>('reset-workspace-size', () => {
    store.dispatch(RootActions.resetSizing());
  });

  await listenGlobal<FocusAction>('focus', (event) => {
    store.dispatch(RootActions.focus(event.payload));
  });

  await listenGlobal<AddWindowPayload>('update-window', (event) => {
    store.dispatch(RootActions.removeWindow(event.payload.hwnd));
    store.dispatch(RootActions.addWindow(event.payload));
  });

  await listenGlobal(SeelenEvent.StateLayoutsChanged, async () => {
    const userSettings = await new UserSettingsLoader().withLayouts().load();
    store.dispatch(RootActions.setAvailableLayouts(userSettings.layouts));
  });

  await StartThemingTool();
}

function loadSettingsCSS(settings: WindowManagerSettings) {
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
