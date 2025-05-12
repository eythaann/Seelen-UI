import { configureStore } from '@reduxjs/toolkit';
import { SeelenCommand, SeelenEvent, Settings, subscribe, UIColors, WegItems } from '@seelen-ui/lib';
import { SeelenWegSettings } from '@seelen-ui/lib/types';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { debounce } from 'lodash';

import { RootActions, RootSlice } from './app';

import { FocusedApp } from '../../../../shared/interfaces/common';
import { StartThemingTool } from '../../../../shared/styles';
import i18n from '../../../i18n';

export const store = configureStore({
  reducer: RootSlice.reducer,
  middleware(getDefaultMiddleware) {
    return getDefaultMiddleware({
      serializableCheck: false,
    });
  },
});

function loadColorsToStore(colors: UIColors) {
  store.dispatch(RootActions.setColors(colors.inner));
}

export async function registerStoreEvents() {
  const view = getCurrentWebviewWindow();

  await view.listen<boolean>(SeelenEvent.WegOverlaped, (event) => {
    store.dispatch(RootActions.setIsOverlaped(event.payload));
  });

  const onFocusChanged = debounce((app: FocusedApp) => {
    store.dispatch(RootActions.setFocusedApp(app));
  }, 200);
  await view.listen<FocusedApp>(SeelenEvent.GlobalFocusChanged, (e) => {
    onFocusChanged(e.payload);
    if (e.payload.name != 'Seelen UI') {
      onFocusChanged.flush();
    }
  });

  await subscribe(SeelenEvent.MediaSessions, (event) => {
    store.dispatch(RootActions.setMediaSessions(event.payload));
  });

  await subscribe(SeelenEvent.Notifications, (event) => {
    store.dispatch(RootActions.setNotifications(event.payload));
  });

  await Settings.onChange(loadSettingsToStore);

  await WegItems.forCurrentWidgetChange(loadWegItemsToStore);

  await UIColors.onChange(loadColorsToStore);

  await StartThemingTool();
}

function loadSettingsCSS(settings: SeelenWegSettings) {
  const styles = document.documentElement.style;

  styles.setProperty('--config-margin', `${settings.margin}px`);
  styles.setProperty('--config-padding', `${settings.padding}px`);

  styles.setProperty('--config-time-before-show', `${settings.delayToShow}ms`);
  styles.setProperty('--config-time-before-hide', `${settings.delayToHide}ms`);

  styles.setProperty('--config-item-size', `${settings.size}px`);
  styles.setProperty('--config-item-zoom-size', `${settings.zoomSize}px`);
  styles.setProperty('--config-space-between-items', `${settings.spaceBetweenItems}px`);
}

function loadSettingsToStore(settings: Settings) {
  i18n.changeLanguage(settings.inner.language || undefined);
  store.dispatch(RootActions.setSettings(settings.seelenweg));
  store.dispatch(RootActions.setDevTools(settings.inner.devTools));
  loadSettingsCSS(settings.seelenweg);
}

function loadWegItemsToStore(items: WegItems) {
  store.dispatch(RootActions.setReorderDisabled(items.inner.isReorderDisabled));
  store.dispatch(RootActions.setItemsOnLeft(items.inner.left));
  store.dispatch(RootActions.setItemsOnCenter(items.inner.center));
  store.dispatch(RootActions.setItemsOnRight(items.inner.right));
}

export async function loadStore() {
  loadSettingsToStore(await Settings.getAsync());
  loadWegItemsToStore(await WegItems.forCurrentWidget());
  loadColorsToStore(await UIColors.getAsync());
  store.dispatch(RootActions.setMediaSessions(await invoke(SeelenCommand.GetMediaSessions)));
}
