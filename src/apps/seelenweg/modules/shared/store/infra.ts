import { UserSettings } from '../../../../../shared.interfaces';
import { loadUserSettings } from '../../../../settings/modules/shared/store/storeApi';
import { loadThemeCSS } from '../../../../shared';
import { Seelenweg, SeelenWegMode, SeelenWegSide } from '../../../../shared/schemas/Seelenweg';
import { updateHitbox } from '../../../events';
import { loadPinnedItems } from './storeApi';
import { configureStore } from '@reduxjs/toolkit';
import { listen as globalListen } from '@tauri-apps/api/event';
import { getCurrent } from '@tauri-apps/api/webviewWindow';

import { PinnedApp } from '../../item/app/PinnedApp';
import { TemporalApp } from '../../item/app/TemporalApp';
import { RootActions, RootSlice } from './app';

import { AppFromBackground, HWND, SavedAppsInYaml } from './domain';

export const store = configureStore({
  reducer: RootSlice.reducer,
});

async function cleanItems(items: AppFromBackground[]): Promise<AppFromBackground[]> {
  const result: AppFromBackground[] = [];
  for (const item of items) {
    const cleaned = await TemporalApp.clean(item);
    result.push(cleaned);
  }
  return result;
}

async function cleanSavedItems(items: SavedAppsInYaml[]): Promise<PinnedApp[]> {
  const result: PinnedApp[] = [];
  for (const item of items) {
    const cleaned = await PinnedApp.clean(item);
    result.push(await PinnedApp.fromSaved(cleaned));
  }
  return result;
}

export async function registerStoreEvents() {
  const view = getCurrent();
  const updateHitboxIfNeeded = () => {
    const { mode } = store.getState().settings;
    if (mode === SeelenWegMode.MIN_CONTENT) {
      updateHitbox();
    }
  };

  await globalListen<UserSettings>('updated-settings', (event) => {
    const userSettings = event.payload;
    const settings = userSettings.jsonSettings.seelenweg;
    store.dispatch(RootActions.setSettings(settings));
    loadSettingsCSS(settings);
    if (userSettings.bgLayers) {
      loadThemeCSS(userSettings);
      store.dispatch(RootActions.setThemeLayers(userSettings.bgLayers));
    }
    updateHitbox();
  });

  await view.listen<AppFromBackground>('add-open-app', async (event) => {
    const item = (await cleanItems([event.payload]))[0]!;
    store.dispatch(RootActions.addOpenApp(item));
    updateHitboxIfNeeded();
  });

  await view.listen<HWND>('remove-open-app', (event) => {
    store.dispatch(RootActions.removeOpenApp(event.payload));
    updateHitboxIfNeeded();
  });

  await view.listen<AppFromBackground>('update-open-app-info', async (event) => {
    const item = (await cleanItems([event.payload]))[0]!;
    store.dispatch(RootActions.updateOpenAppInfo(item));
  });

  await view.listen<AppFromBackground>('replace-open-app', async (event) => {
    const item = (await cleanItems([event.payload]))[0]!;
    store.dispatch(RootActions.addOpenApp(item));
    store.dispatch(RootActions.removeOpenApp(item.process_hwnd));
  });

  await view.listen<HWND>('set-focused-handle', (event) => {
    store.dispatch(RootActions.setFocusedHandle(event.payload));
  });

  await view.listen<boolean>('set-auto-hide', (event) => {
    store.dispatch(RootActions.setIsOverlaped(event.payload));
    updateHitbox();
  });
}

function loadSettingsCSS(settings: Seelenweg) {
  const styles = document.documentElement.style;

  styles.setProperty('--config-margin', `${settings.margin}px`);
  styles.setProperty('--config-padding', `${settings.padding}px`);

  styles.setProperty('--config-item-size', `${settings.size}px`);
  styles.setProperty('--config-item-zoom-size', `${settings.zoomSize}px`);
  styles.setProperty('--config-space-between-items', `${settings.spaceBetweenItems}px`);

  switch (settings.position) {
    case SeelenWegSide.TOP:
      styles.setProperty('--config-by-position-justify-content', 'center');
      styles.setProperty('--config-by-position-align-items', 'flex-start');
      styles.setProperty('--config-by-position-flex-direction', 'row');
      break;
    case SeelenWegSide.BOTTOM:
      styles.setProperty('--config-by-position-justify-content', 'center');
      styles.setProperty('--config-by-position-align-items', 'flex-end');
      styles.setProperty('--config-by-position-flex-direction', 'row');
      break;
    case SeelenWegSide.LEFT:
      styles.setProperty('--config-by-position-justify-content', 'flex-start');
      styles.setProperty('--config-by-position-align-items', 'center');
      styles.setProperty('--config-by-position-flex-direction', 'column');
      break;
    case SeelenWegSide.RIGHT:
      styles.setProperty('--config-by-position-justify-content', 'flex-end');
      styles.setProperty('--config-by-position-align-items', 'center');
      styles.setProperty('--config-by-position-flex-direction', 'column');
      break;
  }
}

export async function loadStore() {
  const userSettings = await loadUserSettings();
  const settings = userSettings.jsonSettings.seelenweg;
  store.dispatch(RootActions.setSettings(settings));
  loadSettingsCSS(settings);

  if (userSettings.bgLayers) {
    loadThemeCSS(userSettings);
    store.dispatch(RootActions.setThemeLayers(userSettings.bgLayers));
  }

  const apps = await loadPinnedItems();
  store.dispatch(RootActions.setPinnedOnLeft(await cleanSavedItems(apps.left) ));
  store.dispatch(RootActions.setPinnedOnCenter(await cleanSavedItems(apps.center )));
  store.dispatch(RootActions.setPinnedOnRight(await cleanSavedItems(apps.right )));
}
