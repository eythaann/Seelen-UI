import { UserSettings } from '../../../../../../shared.interfaces';
import { parseAsCamel } from '../../../../../shared/schemas';
import { IdWithIdentifierSchema } from '../../../../../shared/schemas/AppsConfigurations';
import { ISettings } from '../../../../../shared/schemas/Settings';
import { pick } from 'lodash';

import { AppConfiguration } from '../../../appsConfigurations/domain';
import { RootState } from '../domain';

export const YamlToState_Apps = (yaml: anyObject[]): AppConfiguration[] => {
  const apps: AppConfiguration[] = [];

  yaml.forEach((ymlApp: anyObject) => {
    // filter empty ghost apps used only for add float_identifiers in komorebi cli
    if (ymlApp.options || !ymlApp.float_identifiers) {
      apps.push({
        name: ymlApp.name,
        category: ymlApp.category || null,
        monitor: ymlApp.bound_monitor ?? null,
        workspace: ymlApp.bound_workspace || null,
        identifier: parseAsCamel(IdWithIdentifierSchema, ymlApp.identifier),
        isBundled: ymlApp.is_bundled || false,
        options: ymlApp.options,
      });
    }
  });

  return apps;
};

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

export const StateToJsonSettings = (state: RootState): ISettings => {
  return pick(state, [
    'fancyToolbar',
    'windowManager',
    'seelenweg',
    'selectedTheme',
    'monitors',
    'ahkEnabled',
    'ahkVariables',
    'devTools',
    'language',
    'virtualDesktopStrategy',
  ]);
};
