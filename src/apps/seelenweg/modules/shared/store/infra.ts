import { Theme } from '../../../../../shared.interfaces';
import { updateHitbox } from '../../../events';
import { loadPinnedItems } from './storeApi';
import { configureStore } from '@reduxjs/toolkit';
import { path } from '@tauri-apps/api';
import { convertFileSrc, invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

import { loadUserSettings } from '../../../../settings/modules/shared/infrastructure/storeApi';
import { fs } from '../../../../settings/modules/shared/infrastructure/tauri';
import { getImageBase64FromUrl, getUWPInfoFromExePath } from '../utils/infra';

import { JsonToState_Seelenweg } from '../../../../settings/modules/shared/app/StateBridge';
import { RootActions, RootSlice } from './app';

import { SeelenWegMode, SeelenWegState } from '../../../../settings/modules/seelenweg/domain';
import { AppFromBackground, HWND, PinnedApp } from './domain';

export const store = configureStore({
  reducer: RootSlice.reducer,
});

export type AppDispatch = typeof store.dispatch;
export type store = {
  dispatch: AppDispatch;
  getState: () => {};
};

async function cleanItems(items: AppFromBackground[]) {
  const cleaned: AppFromBackground[] = [];

  for (const item of items) {
    try {
      const uwpInfo = await getUWPInfoFromExePath(item.exe);
      if (uwpInfo && typeof uwpInfo.AppId === 'string') {
        item.execution_path = `shell:AppsFolder\\${uwpInfo.Name}_${uwpInfo.PublisherId}!${uwpInfo.AppId}`;
        const logoPath = uwpInfo.InstallLocation + '\\' + uwpInfo.Logo;
        const logoPath200 = uwpInfo.InstallLocation + '\\' + uwpInfo.Logo.replace('.png', '.scale-200.png');
        const logoPath400 = uwpInfo.InstallLocation + '\\' + uwpInfo.Logo.replace('.png', '.scale-400.png');

        if (await fs.exists(logoPath400)) {
          await fs.copyFile(logoPath400, item.icon);
        } else if (await fs.exists(logoPath200)) {
          await fs.copyFile(logoPath200, item.icon);
        } else if (await fs.exists(logoPath)) {
          await fs.copyFile(logoPath, item.icon);
        }
      }
    } catch (error) {
      console.error('Error while getting UWP info: ', error);
    }

    if (!(await fs.exists(item.icon))) {
      item.icon = await path.resolve(
        await path.resourceDir(),
        'static',
        'icons',
        'missing.png',
      );
    }

    try {
      item.icon = await getImageBase64FromUrl(convertFileSrc(item.icon));
    } catch {
      item.icon = convertFileSrc(item.icon);
    }

    cleaned.push(item);
  }
  return cleaned;
}

export async function registerStoreEvents() {
  const updateHitboxIfNeeded = () => {
    const { mode } = store.getState().settings;
    if (mode === SeelenWegMode.MIN_CONTENT) {
      updateHitbox();
    }
  };

  await listen<AppFromBackground[]>('update-store-apps', async (event) => {
    const items = await cleanItems(event.payload);
    items.forEach((item) => store.dispatch(RootActions.addOpenApp(item)));
    updateHitboxIfNeeded();
  });

  await listen<AppFromBackground>('add-open-app', async (event) => {
    const item = (await cleanItems([event.payload]))[0]!;
    store.dispatch(RootActions.addOpenApp(item));
    updateHitboxIfNeeded();
  });

  await listen<AppFromBackground>('update-open-app', async (event) => {
    const item = (await cleanItems([event.payload]))[0]!;
    store.dispatch(RootActions.updateOpenAppInfo(item));
    updateHitboxIfNeeded();
  });

  await listen<AppFromBackground>('replace-open-app', async (event) => {
    const item = (await cleanItems([event.payload]))[0]!;
    store.dispatch(RootActions.addOpenApp(item));
    store.dispatch(RootActions.removeOpenApp(item.process_hwnd));
  });

  await listen<HWND>('remove-open-app', (event) => {
    store.dispatch(RootActions.removeOpenApp(event.payload));
    updateHitboxIfNeeded();
  });

  await listen<SeelenWegState>('update-store-settings', (event) => {
    document.getElementById('root')!.style.margin = event.payload.margin + 'px';

    store.dispatch(RootActions.setSettings(event.payload));
    updateHitbox();
  });

  await listen<Theme>('update-store-theme', (event) => {
    loadCSSVariables(event.payload);
    store.dispatch(RootActions.setTheme(event.payload));
  });

  invoke('weg_request_apps');
}

function loadCSSVariables(theme: Theme) {
  Object.entries(theme.variables).forEach(([property, value]) => {
    document.documentElement.style.setProperty(property, value);
  });
}

export async function loadStore() {
  const userSettings = await loadUserSettings();
  const initialState = RootSlice.getInitialState();

  const settings = JsonToState_Seelenweg(userSettings.jsonSettings, initialState.settings);
  store.dispatch(RootActions.setSettings(settings));
  if (userSettings.theme) {
    loadCSSVariables(userSettings.theme);
    store.dispatch(RootActions.setTheme(userSettings.theme));
  }

  const apps = await loadPinnedItems();
  store.dispatch(RootActions.setPinnedOnLeft(apps.left as PinnedApp[]));
  store.dispatch(RootActions.setPinnedOnCenter(apps.center as PinnedApp[]));
  store.dispatch(RootActions.setPinnedOnRight(apps.right as PinnedApp[]));
}
