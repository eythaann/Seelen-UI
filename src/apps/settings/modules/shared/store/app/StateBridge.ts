import { UserSettings } from '../../../../../../shared.interfaces';
import { parseAsCamel, VariableConvention } from '../../../../../shared/schemas';
import { IdWithIdentifierSchema } from '../../../../../shared/schemas/AppsConfigurations';
import { ISettings } from '../../../../../shared/schemas/Settings';
import { pick } from 'lodash';

import { AppConfiguration, ApplicationOptions } from '../../../appsConfigurations/domain';
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
        // options
        [ApplicationOptions.Float]: ymlApp.options?.includes(ApplicationOptions.Float) || false,
        [ApplicationOptions.Unmanage]:
          ymlApp.options?.includes(ApplicationOptions.Unmanage) || false,
        [ApplicationOptions.Pinned]: ymlApp.options?.includes(ApplicationOptions.Pinned) || false,
        [ApplicationOptions.ForceManage]:
          ymlApp.options?.includes(ApplicationOptions.ForceManage) || false,
      });
    }
  });

  return apps;
};

export const StaticSettingsToState = (
  userSettings: UserSettings,
  state: RootState,
): RootState => {
  const { jsonSettings, yamlSettings, themes, layouts, placeholders } = userSettings;

  return {
    ...state,
    ...jsonSettings,
    availableThemes: themes,
    availableLayouts: layouts,
    availablePlaceholders: placeholders,
    appsConfigurations: YamlToState_Apps(yamlSettings),
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
  ]);
};

export const StateAppsToYamlApps = (
  appsConfigurations: AppConfiguration[],
  template?: boolean,
): anyObject[] => {
  return appsConfigurations
    .filter((appConfig) => !appConfig.isBundled)
    .map((appConfig: AppConfiguration) => {
      const options = Object.values(ApplicationOptions).filter((option) => appConfig[option]);
      const yamlApp = {
        name: appConfig.name,
        template: template || undefined,
        category: appConfig.category || undefined,
        bound_monitor: appConfig.monitor ?? undefined,
        bound_workspace: appConfig.workspace || undefined,
        identifier: VariableConvention.fromCamelToSnake(appConfig.identifier),
        options: options.length ? options : undefined,
      };
      return yamlApp;
    });
};
