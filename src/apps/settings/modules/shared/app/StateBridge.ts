import { UserSettings } from '../../../../../shared.interfaces';
import { VariableConvention } from './utils';
import { defaultsDeep } from 'lodash';

import {
  AppConfiguration,
  ApplicationIdentifier,
  ApplicationOptions,
  MatchingStrategy,
} from '../../appsConfigurations/domain';
import {
  GeneralSettingsState,
} from '../../general/main/domain';
import { Layout } from '../../monitors/layouts/domain';
import { Monitor, Workspace } from '../../monitors/main/domain';
import { RootState } from '../domain/state';

const JsonToState_Generals = (json: anyObject, generals: GeneralSettingsState): GeneralSettingsState => {
  return {
    selectedTheme: json.selected_theme ?? generals.selectedTheme,
  };
};

export const JsonToState_Monitors = (json: anyObject, monitors: Monitor[]): Monitor[] => {
  if (!json.monitors) {
    return monitors;
  }

  return json.monitors.map((json_monitor: anyObject) => {
    const monitor = Monitor.default();
    const defaultWorkspace = Workspace.default();

    if (json_monitor.work_area_offset) {
      monitor.workAreaOffset = json_monitor.work_area_offset;
    }

    if (json_monitor.workspaces && json_monitor.workspaces.length > 0) {
      monitor.workspaces = json_monitor.workspaces.map((json_workspace: anyObject) => {
        const workspace: Workspace = {
          name: json_workspace.name ?? defaultWorkspace.containerPadding,
          containerPadding: json_workspace.container_padding ?? defaultWorkspace.containerPadding,
          workspacePadding: json_workspace.workspace_padding ?? defaultWorkspace.workspacePadding,
          layout: (json_workspace.layout as Layout) ?? defaultWorkspace.layout,
        };
        return workspace;
      });
    }

    return monitor;
  });
};

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
        matchingStrategy: (ymlApp.identifier.matching_strategy as MatchingStrategy) || MatchingStrategy.Legacy,
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

export const StaticSettingsToState = (userSettings: UserSettings, initialState: RootState): RootState => {
  const { jsonSettings, yamlSettings, ahkEnabled, updateNotification, theme, themes } = userSettings;

  return {
    ...initialState,
    theme,
    availableThemes: themes,
    generals: JsonToState_Generals(jsonSettings, initialState.generals),
    seelenwm: defaultsDeep(VariableConvention.deepKeyParser(jsonSettings.seelen_wm, VariableConvention.snakeToCamel), initialState.seelenwm),
    seelenweg: defaultsDeep(VariableConvention.deepKeyParser(jsonSettings.seelenweg, VariableConvention.snakeToCamel), initialState.seelenweg),
    monitors: JsonToState_Monitors(jsonSettings, initialState.monitors),
    appsConfigurations: YamlToState_Apps(yamlSettings, jsonSettings),
    ahkEnabled,
    updateNotification,
  };
};

export const StateToJsonSettings = (state: RootState): anyObject => {
  return {
    ...VariableConvention.deepKeyParser(state.generals, VariableConvention.camelToSnake),
    monitors: VariableConvention.deepKeyParser(state.monitors, VariableConvention.camelToSnake),
    seelenweg: VariableConvention.deepKeyParser(state.seelenweg, VariableConvention.camelToSnake),
    seelen_wm: VariableConvention.deepKeyParser(state.seelenwm, VariableConvention.camelToSnake),
    seelen_bar: {},
    seelen_shell: {},
  };
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
