import { configureStore } from '@reduxjs/toolkit';
import { listen as listenGlobal } from '@tauri-apps/api/event';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { debounce } from 'lodash';
import { SeelenWegSettings, SeelenWegSide, SwItemType, UIColors, WegItem } from 'seelen-core';

import { SwPinnedAppUtils } from '../../item/app/PinnedApp';
import { SwTemporalAppUtils } from '../../item/app/TemporalApp';
import { RootActions, RootSlice } from './app';

import { AppFromBackground, HWND, MediaSession, SwItem } from './domain';

import { UserSettingsLoader } from '../../../../settings/modules/shared/store/storeApi';
import { FileChange, GlobalEvent } from '../../../../shared/events';
import { FocusedApp } from '../../../../shared/interfaces/common';
import { StartThemingTool } from '../../../../shared/styles';
import i18n from '../../../i18n';
import { IsSavingPinnedItems, loadPinnedItems } from './storeApi';

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

async function cleanSavedItems(items: WegItem[]): Promise<SwItem[]> {
  const result: SwItem[] = [];

  for (const item of items) {
    if (item.type === SwItemType.PinnedApp) {
      result.push(await SwPinnedAppUtils.fromSaved(item));
    } else {
      // TODO remove assert
      result.push(item as SwItem);
    }
  }

  return result;
}

async function initUIColors() {
  function loadColors(colors: UIColors) {
    UIColors.setAssCssVariables(colors);
    store.dispatch(RootActions.setColors(colors));
  }
  loadColors(await UIColors.getAsync());
  await UIColors.onChange(loadColors);
}

export async function registerStoreEvents() {
  const view = getCurrentWebviewWindow();

  await view.listen<boolean>('set-auto-hide', (event) => {
    store.dispatch(RootActions.setIsOverlaped(event.payload));
  });

  await listenGlobal<AppFromBackground[]>('add-multiple-open-apps', async (event) => {
    const items = await cleanItems(event.payload);
    for (const item of items) {
      store.dispatch(RootActions.addOpenApp(item));
    }
  });

  await listenGlobal<AppFromBackground>('add-open-app', async (event) => {
    const item = (await cleanItems([event.payload]))[0]!;
    store.dispatch(RootActions.addOpenApp(item));
  });

  await listenGlobal<HWND>('remove-open-app', (event) => {
    store.dispatch(RootActions.removeOpenApp(event.payload));
  });

  await listenGlobal<AppFromBackground>('update-open-app-info', async (event) => {
    const item = (await cleanItems([event.payload]))[0]!;
    store.dispatch(RootActions.updateOpenAppInfo(item));
  });

  const onFocusChanged = debounce((app: FocusedApp) => {
    store.dispatch(RootActions.setFocusedApp(app));
  }, 200);
  await view.listen<FocusedApp>(GlobalEvent.FocusChanged, (e) => {
    onFocusChanged(e.payload);
    if (e.payload.name != 'Seelen UI') {
      onFocusChanged.flush();
    }
  });

  await listenGlobal<MediaSession[]>('media-sessions', (event) => {
    store.dispatch(RootActions.setMediaSessions(event.payload));
  });

  await initUIColors();

  await listenGlobal<unknown>(FileChange.WegItems, async () => {
    if (IsSavingPinnedItems.current) {
      IsSavingPinnedItems.current = false;
      return;
    }

    const apps = await loadPinnedItems();
    let state = store.getState();

    const leftItems = [
      ...(await cleanSavedItems(apps.left)),
      ...state.itemsOnLeft.filter((item) => item.type === SwItemType.TemporalApp),
    ];

    const centerItems = [
      ...(await cleanSavedItems(apps.center)),
      ...state.itemsOnCenter.filter((item) => item.type === SwItemType.TemporalApp),
    ];

    const rightItems = [
      ...(await cleanSavedItems(apps.right)),
      ...state.itemsOnRight.filter((item) => item.type === SwItemType.TemporalApp),
    ];

    store.dispatch(RootActions.setItemsOnLeft(leftItems));
    store.dispatch(RootActions.setItemsOnCenter(centerItems));
    store.dispatch(RootActions.setItemsOnRight(rightItems));
    await view.emitTo(view.label, 'request-all-open-apps');
  });

  await listenGlobal<any>(FileChange.Settings, async () => {
    await loadSettingsToStore();
  });

  await StartThemingTool();
  await view.emitTo(view.label, 'request-all-open-apps');
}

function loadSettingsCSS(settings: SeelenWegSettings) {
  const styles = document.documentElement.style;

  styles.setProperty('--config-margin', `${settings.margin}px`);
  styles.setProperty('--config-padding', `${settings.padding}px`);

  styles.setProperty('--config-item-size', `${settings.size}px`);
  styles.setProperty('--config-item-zoom-size', `${settings.zoomSize}px`);
  styles.setProperty('--config-space-between-items', `${settings.spaceBetweenItems}px`);

  switch (settings.position) {
    case SeelenWegSide.Top:
      styles.setProperty('--config-by-position-justify-content', 'center');
      styles.setProperty('--config-by-position-align-items', 'flex-start');
      styles.setProperty('--config-by-position-flex-direction', 'row');
      break;
    case SeelenWegSide.Bottom:
      styles.setProperty('--config-by-position-justify-content', 'center');
      styles.setProperty('--config-by-position-align-items', 'flex-end');
      styles.setProperty('--config-by-position-flex-direction', 'row');
      break;
    case SeelenWegSide.Left:
      styles.setProperty('--config-by-position-justify-content', 'flex-start');
      styles.setProperty('--config-by-position-align-items', 'center');
      styles.setProperty('--config-by-position-flex-direction', 'column');
      break;
    case SeelenWegSide.Right:
      styles.setProperty('--config-by-position-justify-content', 'flex-end');
      styles.setProperty('--config-by-position-align-items', 'center');
      styles.setProperty('--config-by-position-flex-direction', 'column');
      break;
  }
}

async function loadSettingsToStore() {
  const userSettings = await new UserSettingsLoader().load();
  i18n.changeLanguage(userSettings.jsonSettings.language);
  const settings = userSettings.jsonSettings.seelenweg;
  store.dispatch(RootActions.setSettings(settings));
  loadSettingsCSS(settings);
}

export async function loadStore() {
  await loadSettingsToStore();
  const apps = await loadPinnedItems();
  store.dispatch(RootActions.setItemsOnLeft(await cleanSavedItems(apps.left)));
  store.dispatch(RootActions.setItemsOnCenter(await cleanSavedItems(apps.center)));
  store.dispatch(RootActions.setItemsOnRight(await cleanSavedItems(apps.right)));
}
