import { UserSettings } from '../../../../../shared.interfaces';
import { UserSettingsLoader } from '../../../../settings/modules/shared/store/storeApi';
import { loadThemeCSS, setColorsAsCssVariables } from '../../../../shared';
import { Seelenweg, SeelenWegMode, SeelenWegSide } from '../../../../shared/schemas/Seelenweg';
import { SwItemType, SwSavedItem } from '../../../../shared/schemas/SeelenWegItems';
import { Theme } from '../../../../shared/schemas/Theme';
import { updateHitbox } from '../../../events';
import i18n from '../../../i18n';
import { IsSavingPinnedItems, loadPinnedItems } from './storeApi';
import { configureStore } from '@reduxjs/toolkit';
import { listen as listenGlobal } from '@tauri-apps/api/event';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { debounce } from 'lodash';

import { SwPinnedAppUtils } from '../../item/app/PinnedApp';
import { SwTemporalAppUtils } from '../../item/app/TemporalApp';
import { RootActions, RootSlice } from './app';

import { AppFromBackground, HWND, MediaSession, SwItem, UIColors } from './domain';

export const store = configureStore({
  reducer: RootSlice.reducer,
});

async function cleanItems(items: AppFromBackground[]): Promise<AppFromBackground[]> {
  const result: AppFromBackground[] = [];
  for (const item of items) {
    const cleaned = await SwTemporalAppUtils.clean(item);
    result.push(cleaned);
  }
  return result;
}

async function cleanSavedItems(items: SwSavedItem[]): Promise<SwItem[]> {
  const result: SwItem[] = [];

  for (const item of items) {
    if (item.type === SwItemType.PinnedApp) {
      result.push(await SwPinnedAppUtils.fromSaved(item));
    } else {
      result.push(item);
    }
  }

  return result;
}

export async function registerStoreEvents() {
  const view = getCurrentWebviewWindow();
  const updateHitboxIfNeeded = () => {
    const { mode } = store.getState().settings;
    if (mode === SeelenWegMode.MIN_CONTENT) {
      updateHitbox();
    }
  };

  await view.listen<boolean>('set-auto-hide', (event) => {
    store.dispatch(RootActions.setIsOverlaped(event.payload));
    updateHitbox();
  });

  await listenGlobal<UserSettings>('updated-settings', (event) => {
    const userSettings = event.payload;
    i18n.changeLanguage(userSettings.jsonSettings.language);
    const settings = userSettings.jsonSettings.seelenweg;
    store.dispatch(RootActions.setSettings(settings));
    loadSettingsCSS(settings);
    loadThemeCSS(userSettings);
    updateHitbox();
  });

  await listenGlobal<AppFromBackground[]>('add-multiple-open-apps', async (event) => {
    const items = await cleanItems(event.payload);
    for (const item of items) {
      store.dispatch(RootActions.addOpenApp(item));
    }
    updateHitboxIfNeeded();
  });

  await listenGlobal<AppFromBackground>('add-open-app', async (event) => {
    const item = (await cleanItems([event.payload]))[0]!;
    store.dispatch(RootActions.addOpenApp(item));
    updateHitboxIfNeeded();
  });

  await listenGlobal<HWND>('remove-open-app', (event) => {
    store.dispatch(RootActions.removeOpenApp(event.payload));
    updateHitboxIfNeeded();
  });

  await listenGlobal<AppFromBackground>('update-open-app-info', async (event) => {
    const item = (await cleanItems([event.payload]))[0]!;
    store.dispatch(RootActions.updateOpenAppInfo(item));
  });

  await listenGlobal<AppFromBackground>('replace-open-app', async (event) => {
    const item = (await cleanItems([event.payload]))[0]!;
    store.dispatch(RootActions.addOpenApp(item));
    store.dispatch(RootActions.removeOpenApp(item.process_hwnd));
  });

  await listenGlobal<HWND>('set-focused-handle', (event) => {
    store.dispatch(RootActions.setFocusedHandle(event.payload));
  });

  await listenGlobal<string>('set-focused-executable', (event) => {
    store.dispatch(RootActions.setFocusedExecutable(event.payload));
  });

  await listenGlobal<MediaSession[]>('media-sessions', (event) => {
    store.dispatch(RootActions.setMediaSessions(event.payload));
  });

  await listenGlobal<UIColors>('colors', (event) => {
    setColorsAsCssVariables(event.payload);
    store.dispatch(RootActions.setColors(event.payload));
  });

  await listenGlobal<Theme[]>('themes', async () => {
    const userSettings = await new UserSettingsLoader().load();
    loadThemeCSS(userSettings);
  });

  await listenGlobal<unknown>(
    'weg-items',
    debounce(async () => {
      if (IsSavingPinnedItems.current) {
        IsSavingPinnedItems.current = false;
        return;
      }

      const apps = await loadPinnedItems();
      store.dispatch(RootActions.setItemsOnLeft(await cleanSavedItems(apps.left)));
      store.dispatch(RootActions.setItemsOnCenter(await cleanSavedItems(apps.center)));
      store.dispatch(RootActions.setItemsOnRight(await cleanSavedItems(apps.right)));
    }, 100),
  );

  await view.emitTo(view.label, 'store-events-ready');
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
  const userSettings = await new UserSettingsLoader().load();
  i18n.changeLanguage(userSettings.jsonSettings.language);

  const settings = userSettings.jsonSettings.seelenweg;
  store.dispatch(RootActions.setSettings(settings));
  loadSettingsCSS(settings);
  loadThemeCSS(userSettings);

  const apps = await loadPinnedItems();
  store.dispatch(RootActions.setItemsOnLeft(await cleanSavedItems(apps.left)));
  store.dispatch(RootActions.setItemsOnCenter(await cleanSavedItems(apps.center)));
  store.dispatch(RootActions.setItemsOnRight(await cleanSavedItems(apps.right)));
}
