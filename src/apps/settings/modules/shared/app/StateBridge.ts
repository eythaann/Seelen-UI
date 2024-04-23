import { UserSettings } from '../../../../../shared.interfaces';
import { SeelenWegMode, SeelenWegSide, SeelenWegState } from '../../../../utils/interfaces/Weg';

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
import { ContainerTopBarMode } from '../../WindowManager/containerTopBar/domain';
import { SeelenManagerState } from '../../WindowManager/main/domain';
import { RootState } from '../domain/state';

const JsonToState_Generals = (json: anyObject, generals: GeneralSettingsState): GeneralSettingsState => {
  return {
    selectedTheme: json.theme_filename ?? generals.selectedTheme,
  };
};

export const JsonToState_WManager = (json: anyObject, wmSettings: SeelenManagerState): SeelenManagerState => {
  const globalWorkAreaOffset = { ...(json.global_work_area_offset ?? wmSettings.globalWorkAreaOffset) };
  globalWorkAreaOffset.bottom = globalWorkAreaOffset.bottom - globalWorkAreaOffset.top;
  globalWorkAreaOffset.right = globalWorkAreaOffset.right - globalWorkAreaOffset.left;

  return {
    enable: json.seelen_wm?.enable ?? wmSettings.enable,
    autoStackinByCategory: json.seelen_wm?.auto_stack_by_category ?? wmSettings.autoStackinByCategory,
    border: {
      enabled: json.seelen_wm?.border?.enabled ?? wmSettings.border.enabled,
      offset: json.seelen_wm?.border?.offset ?? wmSettings.border.offset,
      width: json.seelen_wm?.border?.width ?? wmSettings.border.width,
    },
    containerTopBar: {
      mode: (json.seelen_wm?.top_bar?.mode as ContainerTopBarMode) ?? wmSettings.containerTopBar.mode,
    },
    containerPadding: json.seelen_wm?.default_container_padding ?? wmSettings.containerPadding,
    workspacePadding: json.seelen_wm?.default_workspace_padding ?? wmSettings.workspacePadding,
    globalWorkAreaOffset,
    resizeDelta: json.seelen_wm?.resize_delta ?? wmSettings.resizeDelta,
    floating: {
      width: json.seelen_wm?.floating?.width ?? wmSettings.floating.width,
      height: json.seelen_wm?.floating?.height ?? wmSettings.floating.height,
    },
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

export const JsonToState_Seelenweg = (json: anyObject, initialState: SeelenWegState): SeelenWegState => {
  return {
    enable: json.seelenweg?.enable ?? initialState.enable,
    mode: json.seelenweg?.mode as SeelenWegMode ?? initialState.mode,
    position: json.seelenweg?.position as SeelenWegSide ?? initialState.position,
    size: json.seelenweg?.size ?? initialState.size,
    zoomSize: json.seelenweg?.zoom_size ?? initialState.zoomSize,
    margin: json.seelenweg?.margin ?? initialState.margin,
    padding: json.seelenweg?.padding ?? initialState.padding,
    spaceBetweenItems: json.seelenweg?.space_between_items ?? initialState.spaceBetweenItems,
    visibleSeparators: json.seelenweg?.visible_separators ?? initialState.visibleSeparators,
  };
};

export const StaticSettingsToState = (userSettings: UserSettings, initialState: RootState): RootState => {
  const { jsonSettings, yamlSettings, ahkEnabled, updateNotification, theme, themes } = userSettings;

  return {
    ...initialState,
    theme,
    availableThemes: themes,
    generals: JsonToState_Generals(jsonSettings, initialState.generals),
    seelenwm: JsonToState_WManager(jsonSettings, initialState.seelenwm),
    seelenweg: JsonToState_Seelenweg(jsonSettings, initialState.seelenweg),
    monitors: JsonToState_Monitors(jsonSettings, initialState.monitors),
    appsConfigurations: YamlToState_Apps(yamlSettings, jsonSettings),
    ahkEnabled,
    updateNotification,
  };
};

const StateToJson_WManager = (state: SeelenManagerState): anyObject => {
  const global_work_area_offset = { ...state.globalWorkAreaOffset };
  global_work_area_offset.bottom = global_work_area_offset.bottom + global_work_area_offset.top;
  global_work_area_offset.right = global_work_area_offset.right + global_work_area_offset.left;

  return {
    seelen_wm: {
      auto_stack_by_category: state.autoStackinByCategory,
      border: {
        enabled: state.border.enabled,
        offset: state.border.offset,
        width: state.border.width,
      },
      top_bar: {
        mode: state.containerTopBar.mode,
      },
      default_container_padding: state.containerPadding,
      default_workspace_padding: state.workspacePadding,
      global_work_area_offset: global_work_area_offset as any,
      resize_delta: state.resizeDelta,
      floating: {
        width: state.floating.width,
        height: state.floating.height,
      },
    },
  };
};

const StateToJson_Monitors = (monitors: Monitor[]): anyObject => {
  return {
    monitors: monitors.map((monitor) => {
      return {
        work_area_offset: monitor.workAreaOffset as any || undefined,
        workspaces: monitor.workspaces.map((workspace) => {
          return {
            name: workspace.name,
            container_padding: workspace.containerPadding ?? undefined,
            workspace_padding: workspace.workspacePadding ?? undefined,
            layout: workspace.layout,
          };
        }),
      };
    }),
  };
};

const StateToJson_SeelenWeg = (state: SeelenWegState): anyObject => {
  return {
    seelenweg: {
      enable: state.enable,
      mode: state.mode,
      position: state.position,
      size: state.size,
      zoom_size: state.zoomSize,
      margin: state.margin,
      padding: state.padding,
      space_between_items: state.spaceBetweenItems,
      visible_separators: state.visibleSeparators,
    },
  };
};

const StateToJson_Generals = (state: GeneralSettingsState): anyObject => {
  return {
    theme_filename: state.selectedTheme || undefined,
  };
};

export const StateToJsonSettings = (state: RootState): anyObject => {
  return {
    ...StateToJson_Generals(state.generals),
    ...StateToJson_Monitors(state.monitors),
    ...StateToJson_WManager(state.seelenwm),
    ...StateToJson_SeelenWeg(state.seelenweg),
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
