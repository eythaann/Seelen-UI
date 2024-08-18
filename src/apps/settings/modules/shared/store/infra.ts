import { setColorsAsCssVariables } from '../../../../shared';
import { FileChange } from '../../../../shared/events';
import { parseAsCamel } from '../../../../shared/schemas';
import { SettingsSchema } from '../../../../shared/schemas/Settings';
import { Theme } from '../../../../shared/schemas/Theme';
import { saveUserSettings, UserSettingsLoader } from './storeApi';
import { configureStore } from '@reduxjs/toolkit';
import { emit, listen as listenGlobal } from '@tauri-apps/api/event';
import { Modal } from 'antd';
import { cloneDeep } from 'lodash';

import { startup } from '../tauri/infra';

import { RootActions, RootReducer } from './app/reducer';
import {
  StateAppsToYamlApps,
  StateToJsonSettings,
  StaticSettingsToState,
  YamlToState_Apps,
} from './app/StateBridge';

import { RootState, UIColors } from './domain';

const IsSavingSettings = { current: false };

export const store = configureStore({
  reducer: RootReducer,
});

export type AppDispatch = typeof store.dispatch;
export type store = {
  dispatch: AppDispatch;
  getState: () => RootState;
};

export async function registerStoreEvents() {
  await listenGlobal<any[]>('placeholders', async () => {
    const userSettings = await new UserSettingsLoader().withPlaceholders().load();
    store.dispatch(RootActions.setAvailablePlaceholders(userSettings.placeholders));
  });

  await listenGlobal<Theme[]>('themes', (event) => {
    store.dispatch(RootActions.setAvailableThemes(event.payload));
  });

  await listenGlobal<UIColors>('colors', (event) => {
    setColorsAsCssVariables(event.payload);
    store.dispatch(RootActions.setColors(event.payload));
  });

  await listenGlobal<anyObject[]>('settings-by-app', (event) => {
    store.dispatch(RootActions.setAppsConfigurations(YamlToState_Apps(event.payload)));
  });

  await listenGlobal<any>(FileChange.Settings, (event) => {
    if (IsSavingSettings.current) {
      IsSavingSettings.current = false;
      return;
    }
    const currentState = store.getState();
    const newState: RootState = {
      ...currentState,
      ...parseAsCamel(SettingsSchema, event.payload),
      toBeSaved: false,
    };
    store.dispatch(RootActions.setState(newState));
  });

  await emit('register-colors-events');
}

export const LoadSettingsToStore = async (customPath?: string) => {
  startup.isEnabled().then((value) => {
    store.dispatch(RootActions.setAutostart(value));
  });

  const userSettings = await new UserSettingsLoader()
    .withLayouts()
    .withPlaceholders()
    .withUserApps()
    .withWallpaper()
    .load(customPath);

  const currentState = store.getState();
  const newState = StaticSettingsToState(userSettings, currentState);
  newState.lastLoaded = cloneDeep(newState);
  store.dispatch(RootActions.setState(newState));

  /* // !customPath => avoid start user on manual user loading file
  if (!Object.keys(userSettings.jsonSettings).length && !customPath) {
    StartUser();
  } */
};

export const SaveStore = async () => {
  try {
    const currentState = store.getState();
    const settings = {
      jsonSettings: StateToJsonSettings(currentState),
      yamlSettings: [
        //...StateAppsToYamlApps(currentState.appsTemplates.flatMap((x) => x.apps), true),
        ...StateAppsToYamlApps(currentState.appsConfigurations),
      ],
    };

    IsSavingSettings.current = true;
    await saveUserSettings(settings);
    store.dispatch(RootActions.setToBeSaved(false));
  } catch (error) {
    Modal.error({
      title: 'Error on Save',
      content: String(error),
      centered: true,
    });
  }
};
