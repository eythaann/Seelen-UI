import { UserSettingsLoader } from '../../../../settings/modules/shared/store/storeApi';
import { loadThemeCSS, setColorsAsCssVariables } from '../../../../shared';
import { FileChange } from '../../../../shared/events';
import { WindowManager } from '../../../../shared/schemas/WindowManager';
import { configureStore } from '@reduxjs/toolkit';
import { listen as listenGlobal } from '@tauri-apps/api/event';
import { debounce } from 'lodash';

import { RootActions, RootSlice } from './app';

import { Reservation, Sizing } from '../../layout/domain';
import { AddWindowPayload, DesktopId, FocusAction, UIColors } from './domain';

export const store = configureStore({
  reducer: RootSlice.reducer,
  devTools: true,
});

export async function loadStore() {
  const userSettings = await new UserSettingsLoader().withLayouts().load();
  const settings = userSettings.jsonSettings.windowManager;
  store.dispatch(RootActions.setAvailableLayouts(userSettings.layouts));
  store.dispatch(RootActions.setSettings(settings));
  loadSettingsCSS(settings);
  loadThemeCSS(userSettings);
}

export async function registerStoreEvents() {
  await listenGlobal<any>(
    FileChange.Settings,
    debounce(async () => {
      await loadStore();
    }, 100),
  );

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

  await listenGlobal<UIColors>('colors', (event) => {
    setColorsAsCssVariables(event.payload);
    store.dispatch(RootActions.setColors(event.payload));
  });

  await listenGlobal(
    FileChange.Themes,
    debounce(async () => {
      const userSettings = await new UserSettingsLoader().load();
      loadThemeCSS(userSettings);
    }, 100),
  );

  await listenGlobal(
    FileChange.Placeholders,
    debounce(async () => {
      const userSettings = await new UserSettingsLoader().withLayouts().load();
      store.dispatch(RootActions.setAvailableLayouts(userSettings.layouts));
    }, 100),
  );
}

function loadSettingsCSS(settings: WindowManager) {
  const styles = document.documentElement.style;

  styles.setProperty('--config-padding', `${settings.workspacePadding}px`);
  styles.setProperty('--config-containers-gap', `${settings.workspaceGap}px`);

  styles.setProperty('--config-margin-top', `${settings.globalWorkAreaOffset.top}px`);
  styles.setProperty('--config-margin-left', `${settings.globalWorkAreaOffset.left}px`);
  styles.setProperty('--config-margin-right', `${settings.globalWorkAreaOffset.right}px`);
  styles.setProperty('--config-margin-bottom', `${settings.globalWorkAreaOffset.bottom}px`);

  styles.setProperty('--config-border-offset', `${settings.border.offset}px`);
  styles.setProperty('--config-border-width', `${settings.border.width}px`);
}
