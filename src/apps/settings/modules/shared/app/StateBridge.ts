import { StaticConfig } from '../../../../../JsonSettings.interface';
import { UserSettings } from '../../../../../shared.interfaces';
import { ApplicationConfiguration as YamlAppConfiguration } from '../../../../../YamlSettings.interface';
import { Rect } from './Rect';
import { ColorFactory } from 'antd/es/color-picker/color';

import {
  AppConfiguration,
  ApplicationIdentifier,
  ApplicationOptions,
  MatchingStrategy,
} from '../../appsConfigurations/domain';
import { ContainerTopBarMode } from '../../general/containerTopBar/domain';
import {
  CrossMonitorMoveBehaviour,
  FocusFollowsMouse,
  GeneralSettingsState,
  UnmanagedWindowOperationBehaviour,
  WindowContainerBehaviour,
  WindowHidingBehaviour,
} from '../../general/main/domain';
import { Layout } from '../../monitors/layouts/domain';
import { Monitor, Workspace } from '../../monitors/main/domain';
import { SeelenWegMode, SeelenWegState } from '../../seelenweg/domain';
import { HexColor } from '../domain/interfaces';
import { RootState } from '../domain/state';

const JsonToState_Generals = (json: StaticConfig, generals: GeneralSettingsState): GeneralSettingsState => {
  return {
    selectedTheme: json.theme_filename ?? generals.selectedTheme,
    altFocusHack: !!json.alt_focus_hack,
    animations: {
      finishMiminization: json.animations?.finish_miminization_before_restore ?? generals.animations.finishMiminization,
      nativeDelay: json.animations?.native_animations_delay ?? generals.animations.nativeDelay,
    },
    autoStackinByCategory: json.auto_stack_by_category ?? generals.autoStackinByCategory,
    border: {
      enable: json.active_window_border ?? generals.border.enable,
      colorSingle: new ColorFactory(
        json.active_window_border_colours?.single || generals.border.colorSingle,
      ).toHexString() as HexColor,
      colorMonocle: new ColorFactory(
        json.active_window_border_colours?.monocle || generals.border.colorMonocle,
      ).toHexString() as HexColor,
      colorStack: new ColorFactory(
        json.active_window_border_colours?.stack || generals.border.colorStack,
      ).toHexString() as HexColor,
      offset: json.border_offset ?? generals.border.offset,
      width: json.border_width ?? generals.border.width,
    },
    popups: {
      enable: json.popups?.enable ?? generals.popups.enable,
      x: json.popups?.x ?? generals.popups.x,
      y: json.popups?.y ?? generals.popups.y,
      width: json.popups?.width ?? generals.popups.width,
      height: json.popups?.height ?? generals.popups.height,
      textColor: new ColorFactory(json.popups?.text_color ?? generals.popups.textColor).toHexString(),
      background: new ColorFactory(json.popups?.background ?? generals.popups.background).toHexString(),
      borderColor: new ColorFactory(json.popups?.border_color ?? generals.popups.borderColor).toHexString(),
      borderWidth: json.popups?.border_width ?? generals.popups.borderWidth,
    },
    crossMonitorMoveBehaviour:
      (json.cross_monitor_move_behaviour as CrossMonitorMoveBehaviour) ?? generals.crossMonitorMoveBehaviour,

    containerTopBar: {
      height: json.top_bar?.height ?? generals.containerTopBar.height,
      mode: (json.top_bar?.mode as ContainerTopBarMode) ?? generals.containerTopBar.mode,
      tabs: {
        width: json.top_bar?.tabs?.width ?? generals.containerTopBar.tabs.width,
        color: new ColorFactory(
          json.top_bar?.tabs?.color || generals.containerTopBar.tabs.color,
        ).toHexString() as HexColor,
        background: new ColorFactory(
          json.top_bar?.tabs?.background || generals.containerTopBar.tabs.background,
        ).toHexString() as HexColor,
      },
    },
    containerPadding: json.default_container_padding ?? generals.containerPadding,
    workspacePadding: json.default_workspace_padding ?? generals.workspacePadding,
    focusFollowsMouse: (json.focus_follows_mouse as FocusFollowsMouse) ?? generals.focusFollowsMouse,
    globalWorkAreaOffset: json.global_work_area_offset ?? generals.globalWorkAreaOffset,
    monitorIndexPreferences:
      (json.monitor_index_preferences as Record<string, Rect>) ?? generals.monitorIndexPreferences,
    displayindexpreferences: json.display_index_preferences ?? generals.displayindexpreferences,
    mouseFollowFocus: json.mouse_follows_focus ?? generals.mouseFollowFocus,
    resizeDelta: json.resize_delta ?? generals.resizeDelta,
    unmanagedWindowOperationBehaviour:
      (json.unmanaged_window_operation_behaviour as UnmanagedWindowOperationBehaviour) ??
      generals.unmanagedWindowOperationBehaviour,
    windowContainerBehaviour:
      (json.window_container_behaviour as WindowContainerBehaviour) ?? generals.windowContainerBehaviour,
    windowHidingBehaviour: (json.window_hiding_behaviour as WindowHidingBehaviour) ?? generals.windowHidingBehaviour,
  };
};

const JsonToState_Monitors = (json: StaticConfig, monitors: Monitor[]): Monitor[] => {
  if (!json.monitors) {
    return monitors;
  }

  return json.monitors.map((json_monitor) => {
    const monitor = Monitor.default();
    const defaultWorkspace = Workspace.default();

    if (json_monitor.work_area_offset) {
      monitor.workAreaOffset = json_monitor.work_area_offset;
    }

    if (json_monitor.workspaces && json_monitor.workspaces.length > 0) {
      monitor.workspaces = json_monitor.workspaces.map<Workspace>((json_workspace) => {
        const workspace: Workspace = {
          name: json_workspace.name ?? defaultWorkspace.containerPadding,
          containerPadding: json_workspace.container_padding ?? defaultWorkspace.containerPadding,
          workspacePadding: json_workspace.workspace_padding ?? defaultWorkspace.workspacePadding,
          customLayout: json_workspace.custom_layout ?? defaultWorkspace.customLayout,
          customLayoutRules: json_workspace.custom_layout_rules ?? defaultWorkspace.customLayoutRules,
          layout: (json_workspace.layout as Layout) ?? defaultWorkspace.layout,
          layoutRules: (json_workspace.layout_rules as Record<string, Layout>) ?? defaultWorkspace.layoutRules,
        };
        return workspace;
      });
    }

    return monitor;
  });
};

export const YamlToState_Apps = (yaml: YamlAppConfiguration[], json: StaticConfig = {}): AppConfiguration[] => {
  const apps: AppConfiguration[] = [];

  yaml.forEach((ymlApp: YamlAppConfiguration) => {
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
        /*[ApplicationOptions.BorderOverflow]: ymlApp.options?.includes('border_overflow') || false,*/
        [ApplicationOptions.Force]: ymlApp.options?.includes('force') || false,
        [ApplicationOptions.Layered]: ymlApp.options?.includes('layered') || false,
        [ApplicationOptions.ObjectNameChange]: ymlApp.options?.includes('object_name_change') || false,
        [ApplicationOptions.TrayAndMultiWindow]: ymlApp.options?.includes('tray_and_multi_window') || false,
        [ApplicationOptions.Unmanage]: ymlApp.options?.includes('unmanage') || false,
      });
    }

    // In komorebi cli float_identifiers are considerated as unmanaged
    // also we doesn't use this object whe use float option instead
    ymlApp.float_identifiers?.forEach((rule) => {
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

  if (json.manage_rules) {
    Object.values(json.manage_rules).forEach((rule) => {
      apps.push({
        ...AppConfiguration.from(rule),
        [ApplicationOptions.Force]: true,
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

  if (json.object_name_change_applications) {
    Object.values(json.object_name_change_applications).forEach((rule) => {
      apps.push({
        ...AppConfiguration.from(rule),
        [ApplicationOptions.ObjectNameChange]: true,
      });
    });
  }

  if (json.layered_applications) {
    Object.values(json.layered_applications).forEach((rule) => {
      apps.push({
        ...AppConfiguration.from(rule),
        [ApplicationOptions.Layered]: true,
      });
    });
  }

  if (json.tray_and_multi_window_applications) {
    Object.values(json.tray_and_multi_window_applications).forEach((rule) => {
      apps.push({
        ...AppConfiguration.from(rule),
        [ApplicationOptions.TrayAndMultiWindow]: true,
      });
    });
  }

  if (json.exclude_float_rules) {
    Object.values(json.exclude_float_rules).forEach((rule) => {
      apps.push({
        ...AppConfiguration.from(rule),
        // force disable float rules on komorebi
        [ApplicationOptions.Force]: true,
      });
    });
  }

  json.monitors?.forEach(({ workspaces }, monitor_idx) => {
    workspaces?.forEach(({ workspace_rules, name }) => {
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

export const JsonToState_Seelenweg = (json: StaticConfig, initialState: SeelenWegState): SeelenWegState => {
  return {
    enabled: json.seelenweg?.enabled ?? initialState.enabled,
    mode: json.seelenweg?.mode as SeelenWegMode ?? initialState.mode,
    size: json.seelenweg?.size ?? initialState.size,
    zoomSize: json.seelenweg?.zoom_size ?? initialState.zoomSize,
    margin: json.seelenweg?.margin ?? initialState.margin,
    padding: json.seelenweg?.padding ?? initialState.padding,
    spaceBetweenItems: json.seelenweg?.space_between_items ?? initialState.spaceBetweenItems,
  };
};

export const StaticSettingsToState = (userSettings: UserSettings, initialState: RootState): RootState => {
  const { jsonSettings, yamlSettings, ahkEnabled, updateNotification, theme, themes } = userSettings;

  return {
    ...initialState,
    theme,
    availableThemes: themes,
    generals: JsonToState_Generals(jsonSettings, initialState.generals),
    seelenweg: JsonToState_Seelenweg(jsonSettings, initialState.seelenweg),
    monitors: JsonToState_Monitors(jsonSettings, initialState.monitors),
    appsConfigurations: YamlToState_Apps(yamlSettings, jsonSettings),
    ahkEnabled,
    updateNotification,
  };
};

const StateToJson_Generals = (state: GeneralSettingsState): StaticConfig => {
  return {
    theme_filename: state.selectedTheme,
    alt_focus_hack: state.altFocusHack,
    animations: {
      finish_miminization_before_restore: state.animations.finishMiminization,
      native_animations_delay: state.animations.nativeDelay,
    },
    auto_stack_by_category: state.autoStackinByCategory,
    active_window_border: state.border.enable,
    active_window_border_colours: {
      single: new ColorFactory(state.border.colorSingle).toRgb(),
      monocle: new ColorFactory(state.border.colorMonocle).toRgb(),
      stack: new ColorFactory(state.border.colorStack).toRgb(),
    },
    border_offset: state.border.offset,
    border_width: state.border.width,
    cross_monitor_move_behaviour: state.crossMonitorMoveBehaviour,
    top_bar: {
      height: state.containerTopBar.height,
      mode: state.containerTopBar.mode,
      tabs: {
        width: state.containerTopBar.tabs.width,
        color: state.containerTopBar.tabs.color,
        background: state.containerTopBar.tabs.background,
      },
    },
    popups: {
      enable: state.popups.enable,
      x: state.popups.x,
      y: state.popups.y,
      width: state.popups.width,
      height: state.popups.height,
      text_color: state.popups.textColor,
      background: state.popups.background,
      border_color: state.popups.borderColor,
      border_width: state.popups.borderWidth,
    },
    default_container_padding: state.containerPadding,
    default_workspace_padding: state.workspacePadding,
    focus_follows_mouse: state.focusFollowsMouse ?? undefined,
    global_work_area_offset: state.globalWorkAreaOffset as any,
    mouse_follows_focus: state.mouseFollowFocus,
    resize_delta: state.resizeDelta,
    unmanaged_window_operation_behaviour: state.unmanagedWindowOperationBehaviour,
    window_container_behaviour: state.windowContainerBehaviour,
    window_hiding_behaviour: state.windowHidingBehaviour,
  };
};

const cleanRules = <T = string>(rules: Record<string, string | null> | null): Record<string, T> | undefined => {
  const cleanedRules = { ...rules };
  for (const key of Object.keys(cleanedRules)) {
    if (cleanedRules[key] == null || cleanedRules[key] === '') {
      delete cleanedRules[key];
    }
  }
  return Object.keys(cleanedRules).length ? (cleanedRules as Record<string, T>) : undefined;
};

const StateToJson_Monitors = (monitors: Monitor[]): Partial<StaticConfig> => {
  return {
    monitors: monitors.map((monitor) => {
      return {
        work_area_offset: monitor.workAreaOffset as any || undefined,
        workspaces: monitor.workspaces.map((workspace) => {
          return {
            name: workspace.name,
            container_padding: workspace.containerPadding ?? undefined,
            workspace_padding: workspace.workspacePadding ?? undefined,
            custom_layout: workspace.customLayout || undefined,
            custom_layout_rules: cleanRules(workspace.customLayoutRules),
            layout: workspace.layout,
            layout_rules: cleanRules(workspace.layoutRules),
          };
        }),
      };
    }),
  };
};

const StateToJson_SeelenWeg = (state: SeelenWegState): Partial<StaticConfig> => {
  return {
    seelenweg: {
      enabled: state.enabled,
      mode: state.mode,
      size: state.size,
      zoom_size: state.zoomSize,
      margin: state.margin,
      padding: state.padding,
      space_between_items: state.spaceBetweenItems,
    },
  };
};

export const StateToJsonSettings = (state: RootState): StaticConfig => {
  return {
    ...StateToJson_Generals(state.generals),
    ...StateToJson_Monitors(state.monitors),
    ...StateToJson_SeelenWeg(state.seelenweg),
  };
};

export const StateAppsToYamlApps = (
  appsConfigurations: AppConfiguration[],
  template?: boolean,
): YamlAppConfiguration[] => {
  return appsConfigurations.map((appConfig: AppConfiguration) => {
    const options = Object.values(ApplicationOptions).filter((option) => appConfig[option]);
    const yamlApp: YamlAppConfiguration = {
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
