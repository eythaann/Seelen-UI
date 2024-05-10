import { UserSettings } from '../../../../../../shared.interfaces';
import { ISettings } from '../../../../../utils/schemas/Settings';
import { pick } from 'lodash';

import {
  AppConfiguration,
  ApplicationIdentifier,
  ApplicationOptions,
  MatchingStrategy,
} from '../../../appsConfigurations/domain';
import { RootState } from '../domain';

export const YamlToState_Apps = (yaml: anyObject[], json?: ISettings): AppConfiguration[] => {
  const apps: AppConfiguration[] = [];

  yaml.forEach((ymlApp: anyObject) => {
    if (ymlApp.template) {
      return;
    }

    // filter empty ghost apps used only for add float_identifiers in komorebi cli
    if (ymlApp.options || !ymlApp.float_identifiers) {
      apps.push({
        name: ymlApp.name,
        category: ymlApp.category || null,
        monitor: ymlApp.bound_monitor ?? null,
        workspace: ymlApp.bound_workspace || null,
        identifier: ymlApp.identifier.id,
        kind: ymlApp.identifier.kind as ApplicationIdentifier,
        matchingStrategy:
          (ymlApp.identifier.matching_strategy as MatchingStrategy) || MatchingStrategy.Legacy,
        // options
        [ApplicationOptions.Float]: ymlApp.options?.includes(ApplicationOptions.Float) || false,
        [ApplicationOptions.Unmanage]: ymlApp.options?.includes(ApplicationOptions.Unmanage) || false,
        [ApplicationOptions.Pinned]: ymlApp.options?.includes(ApplicationOptions.Pinned) || false,
        [ApplicationOptions.ForceManage]: ymlApp.options?.includes(ApplicationOptions.ForceManage) || false,
      });
    }

    // In komorebi cli float_identifiers are considerated as unmanaged
    // also we doesn't use this object whe use float option instead
    ymlApp.float_identifiers?.forEach((rule: any) => {
      apps.push({
        ...AppConfiguration.from(rule),
        [ApplicationOptions.Unmanage]: true,
      });
    });
  });

  json?.monitors?.forEach(({ workspaces }: anyObject, monitor_idx: number) => {
    workspaces?.forEach(({ workspace_rules, name }: anyObject) => {
      if (!workspace_rules) {
        return;
      }
      Object.values(workspace_rules).forEach((rule) => {
        apps.push({
          ...AppConfiguration.from(rule),
          monitor: monitor_idx,
          workspace: name,
        });
      });
    });
  });

  return apps;
};

export const StaticSettingsToState = (
  userSettings: UserSettings,
  initialState: RootState,
): RootState => {
  const { jsonSettings, yamlSettings, theme, themes, layouts, placeholders } = userSettings;

  return {
    ...initialState,
    ...jsonSettings,
    selectedTheme: theme?.info.filename || null,
    availableThemes: themes,
    availableLayouts: layouts,
    availablePlaceholders: placeholders,
    appsConfigurations: YamlToState_Apps(yamlSettings, jsonSettings),
  };
};

export const StateToJsonSettings = (state: RootState): ISettings => {
  return pick(state, ['windowManager', 'seelenweg', 'monitors', 'selectedTheme', 'ahkEnabled', 'fancyToolbar', 'ahkVariables']);
};

export const StateAppsToYamlApps = (
  appsConfigurations: AppConfiguration[],
  template?: boolean,
): anyObject[] => {
  return appsConfigurations.map((appConfig: AppConfiguration) => {
    const options = Object.values(ApplicationOptions).filter((option) => appConfig[option]);
    const yamlApp = {
      name: appConfig.name,
      template: template || undefined,
      category: appConfig.category || undefined,
      bound_monitor: appConfig.monitor ?? undefined,
      bound_workspace: appConfig.workspace || undefined,
      identifier: {
        id: appConfig.identifier,
        kind: appConfig.kind,
        matching_strategy: appConfig.matchingStrategy,
      },
      options: options.length ? options : undefined,
    };
    return yamlApp;
  });
};
