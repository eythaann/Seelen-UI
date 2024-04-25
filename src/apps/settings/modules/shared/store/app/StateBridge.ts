import { UserSettings } from '../../../../../../shared.interfaces';
import { defaultsDeep, pick } from 'lodash';

import { VariableConvention } from '../../utils/app';

import {
  AppConfiguration,
  ApplicationIdentifier,
  ApplicationOptions,
  MatchingStrategy,
} from '../../../appsConfigurations/domain';
import { Monitor } from '../../../monitors/main/domain';
import { RootState } from '../domain';

export const YamlToState_Apps = (yaml: anyObject[], json: anyObject = {}): AppConfiguration[] => {
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
        monitor: ymlApp.binded_monitor ?? null,
        workspace: ymlApp.binded_workspace || null,
        identifier: ymlApp.identifier.id,
        kind: ymlApp.identifier.kind as ApplicationIdentifier,
        matchingStrategy:
          (ymlApp.identifier.matching_strategy as MatchingStrategy) || MatchingStrategy.Legacy,
        // options
        [ApplicationOptions.Float]: ymlApp.options?.includes('float') || false,
        [ApplicationOptions.Unmanage]: ymlApp.options?.includes('unmanage') || false,
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

  /**
   * From here are just migration from komorebi cli static configs.
   */
  if (json.unmanage_rules) {
    Object.values(json.unmanage_rules).forEach((rule) => {
      apps.push({
        ...AppConfiguration.from(rule),
        [ApplicationOptions.Unmanage]: true,
      });
    });
  }

  if (json.float_rules) {
    Object.values(json.float_rules).forEach((rule) => {
      apps.push({
        ...AppConfiguration.from(rule),
        [ApplicationOptions.Float]: true,
      });
    });
  }

  json.monitors?.forEach(({ workspaces }: anyObject, monitor_idx: number) => {
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
  const { jsonSettings, yamlSettings, ahkEnabled, theme, themes } = userSettings;

  return {
    ...initialState,
    theme,
    availableThemes: themes,
    windowManager: defaultsDeep(
      VariableConvention.fromSnakeToCamel(jsonSettings.seelen_wm),
      initialState.windowManager,
    ),
    seelenweg: defaultsDeep(
      VariableConvention.fromSnakeToCamel(jsonSettings.seelenweg),
      initialState.seelenweg,
    ),
    monitors: jsonSettings.monitors
      ? (VariableConvention.fromSnakeToCamel(jsonSettings.monitors) as Monitor[])
      : initialState.monitors,
    appsConfigurations: YamlToState_Apps(yamlSettings, jsonSettings),
    ahkEnabled,
  };
};

export const StateToJsonSettings = (state: RootState): anyObject => {
  return VariableConvention.fromCamelToSnake(
    pick(state, ['windowManager', 'seelenweg', 'monitors', 'selectedTheme', 'ahkEnabled']),
  );
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
      binded_monitor: appConfig.monitor ?? undefined,
      binded_workspace: appConfig.workspace || undefined,
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
