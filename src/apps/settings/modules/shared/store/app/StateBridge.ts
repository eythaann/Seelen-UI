import { Settings } from '@seelen-ui/lib/types';
import { pick } from 'lodash';

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
    'iconPacks',
    'selectedThemes',
    'monitorsV2',
    'ahkEnabled',
    'ahkVariables',
    'devTools',
    'language',
    'dateFormat',
    'virtualDesktopStrategy',
    'updater',
    'wall',
    'launcher',
    'custom',
  ]);
};
