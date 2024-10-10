import { pick } from 'lodash';
import { Settings } from 'seelen-core';

import { RootState } from '../domain';

import { UserSettings } from '../../../../../../shared.interfaces';

export const StaticSettingsToState = (userSettings: UserSettings, state: RootState): RootState => {
  const { jsonSettings, yamlSettings, themes, layouts, placeholders, wallpaper } = userSettings;
  return {
    ...state,
    ...jsonSettings,
    wallpaper,
    availableThemes: themes,
    availableLayouts: layouts,
    availablePlaceholders: placeholders,
    appsConfigurations: yamlSettings,
  };
};

export const StateToJsonSettings = (state: RootState): Settings => {
  return pick(state, [
    'fancyToolbar',
    'windowManager',
    'seelenweg',
    'selectedThemes',
    'monitors',
    'ahkEnabled',
    'ahkVariables',
    'devTools',
    'language',
    'dateFormat',
    'virtualDesktopStrategy',
    'betaChannel',
    'wall',
    'launcher',
  ]);
};
