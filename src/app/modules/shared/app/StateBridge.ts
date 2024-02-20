import { StaticConfig } from '../../../../JsonSettings.interface';
import { ColorFactory } from 'antd/es/color-picker/color';

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
import { HexColor } from '../domain/interfaces';
import { RootState } from '../domain/state';

const JsonToState_Generals = (json: StaticConfig, generals: GeneralSettingsState): GeneralSettingsState => {
  return {
    altFocusHack: json.alt_focus_hack ?? generals.altFocusHack,
    animations: {
      finishMiminization: json.animations?.finish_miminization_before_restore ?? generals.animations.finishMiminization,
      nativeDelay: json.animations?.native_animations_delay ?? generals.animations.nativeDelay,
    },
    autoStackinByCategory: json.auto_stack_by_category ?? generals.autoStackinByCategory,
    border: {
      enable: json.active_window_border ?? generals.border.enable,
      color: new ColorFactory(json.active_window_border_colours?.single || generals.border.color).toHex() as HexColor,
      offset: json.active_window_border_offset ?? generals.border.offset,
      width: json.active_window_border_width ?? generals.border.width,
    },
    crossMonitorMoveBehaviour:
      (json.cross_monitor_move_behaviour as CrossMonitorMoveBehaviour) ?? generals.crossMonitorMoveBehaviour,

    containerTopBar: {
      height: json.top_bar?.height ?? generals.containerTopBar.height,
      mode: (json.top_bar?.mode as ContainerTopBarMode) ?? generals.containerTopBar.mode,
      tabs: {
        width: json.top_bar?.tabs?.width ?? generals.containerTopBar.tabs.width,
        color: new ColorFactory(json.top_bar?.tabs?.color || generals.containerTopBar.tabs.color).toHex() as HexColor,
        background: new ColorFactory(
          json.top_bar?.tabs?.background || generals.containerTopBar.tabs.background,
        ).toHex() as HexColor,
      },
    },
    containerPadding: json.default_container_padding ?? generals.containerPadding,
    workspacePadding: json.default_workspace_padding ?? generals.workspacePadding,
    focusFollowsMouse: (json.focus_follows_mouse as FocusFollowsMouse) ?? generals.focusFollowsMouse,
    globalWorkAreaOffset: json.global_work_area_offset ?? generals.globalWorkAreaOffset,
    invisibleBorders: json.invisible_borders ?? generals.invisibleBorders,
    //maybe a todo: monitorIndexPreferences: json.monitor_index_preferences ?? generals.monitorIndexPreferences,
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
          layout: json_workspace.layout as Layout ?? defaultWorkspace.layout,
          layoutRules: json_workspace.layout_rules as Record<string, Layout> ?? defaultWorkspace.layoutRules,
        };
        return workspace;
      });
    }

    return monitor;
  });
};

export const JsonToState = (json: StaticConfig, initialState: RootState): RootState => {
  return {
    route: initialState.route,
    toBeSaved: false,
    generals: JsonToState_Generals(json, initialState.generals),
    monitors: JsonToState_Monitors(json, initialState.monitors), //TODO
  };
};
