import { EventHandler, InvokeHandler, Obtainable } from '../handlers';
import { Rect } from '../utils';

export enum VirtualDesktopStrategy {
  Native = 'Native',
  Seelen = 'Seelen',
}

export enum SeelenWegMode {
  FullWidth = 'Full-Width',
  MinContent = 'Min-Content',
}

export enum HideMode {
  Never = 'Never',
  Always = 'Always',
  OnOverlap = 'On-Overlap',
}

export enum SeelenWegSide {
  Left = 'Left',
  Right = 'Right',
  Top = 'Top',
  Bottom = 'Bottom',
}

export class SeelenWallSettings {
  enabled: boolean = true;
}

export class Settings extends Obtainable<Settings>(
  InvokeHandler.GetSettings,
  EventHandler.Settings,
) {
  fancyToolbar: FancyToolbarSettings = new FancyToolbarSettings();
  seelenweg: SeelenWegSettings = new SeelenWegSettings();
  windowManager: WindowManagerSettings = new WindowManagerSettings();
  wall: SeelenWallSettings = new SeelenWallSettings();
  monitors: Monitor[] = [new Monitor()];
  ahkEnabled: boolean = true;
  ahkVariables: AhkVarList = new AhkVarList();
  selectedTheme: string[] = ['default'];
  devTools: boolean = false;
  language: string = '';
  virtualDesktopStrategy: VirtualDesktopStrategy = VirtualDesktopStrategy.Native;
  betaChannel: boolean = false;
}

export class FancyToolbarSettings {
  enabled: boolean = true;
  height: number = 30;
  placeholder: string = 'default.yml';
  hideMode: HideMode = HideMode.Never;
}

export class SeelenWegSettings {
  enabled: boolean = true;
  mode: SeelenWegMode = SeelenWegMode.MinContent;
  hideMode: HideMode = HideMode.OnOverlap;
  position: SeelenWegSide = SeelenWegSide.Bottom;
  visibleSeparators: boolean = true;
  size: number = 40;
  zoomSize: number = 70;
  margin: number = 8;
  padding: number = 8;
  spaceBetweenItems: number = 8;
}

export class Border {
  enabled: boolean = true;
  width: number = 3.0;
  offset: number = 0.0;
}

export class FloatingWindowSettings {
  width: number = 800.0;
  height: number = 500.0;
}

export class WindowManagerSettings {
  enabled: boolean = false;
  autoStackingByCategory: boolean = true;
  border: Border = new Border();
  resizeDelta: number = 10.0;
  workspaceGap: number = 10.0;
  workspacePadding: number = 10.0;
  globalWorkAreaOffset: Rect = new Rect();
  floating: FloatingWindowSettings = new FloatingWindowSettings();
  defaultLayout: string = 'default.yml';
}

export class Workspace {
  name: string = 'New Workspace';
  layout: string = 'BSP';
  padding: number | null = null;
  gap: number | null = null;
}

export class Monitor {
  workspaces: Workspace[] = [new Workspace()];
  workAreaOffset: Rect | null = null;
  /** This is only used internally on settings, rust provider does not contain this key */
  editingWorkspace?: number;
}

export class AhkVar {
  fancy: string = '';
  ahk: string = '';
}

export class AhkVarList {
  reserve_top = new AhkVar();
  reserve_bottom = new AhkVar();
  reserve_left = new AhkVar();
  reserve_right = new AhkVar();
  reserve_float = new AhkVar();
  reserve_stack = new AhkVar();
  focus_top = new AhkVar();
  focus_bottom = new AhkVar();
  focus_left = new AhkVar();
  focus_right = new AhkVar();
  focus_latest = new AhkVar();
  increase_width = new AhkVar();
  decrease_width = new AhkVar();
  increase_height = new AhkVar();
  decrease_height = new AhkVar();
  restore_sizes = new AhkVar();
  switch_workspace_0 = new AhkVar();
  switch_workspace_1 = new AhkVar();
  switch_workspace_2 = new AhkVar();
  switch_workspace_3 = new AhkVar();
  switch_workspace_4 = new AhkVar();
  switch_workspace_5 = new AhkVar();
  switch_workspace_6 = new AhkVar();
  switch_workspace_7 = new AhkVar();
  switch_workspace_8 = new AhkVar();
  switch_workspace_9 = new AhkVar();
  move_to_workspace_0 = new AhkVar();
  move_to_workspace_1 = new AhkVar();
  move_to_workspace_2 = new AhkVar();
  move_to_workspace_3 = new AhkVar();
  move_to_workspace_4 = new AhkVar();
  move_to_workspace_5 = new AhkVar();
  move_to_workspace_6 = new AhkVar();
  move_to_workspace_7 = new AhkVar();
  move_to_workspace_8 = new AhkVar();
  move_to_workspace_9 = new AhkVar();
  send_to_workspace_0 = new AhkVar();
  send_to_workspace_1 = new AhkVar();
  send_to_workspace_2 = new AhkVar();
  send_to_workspace_3 = new AhkVar();
  send_to_workspace_4 = new AhkVar();
  send_to_workspace_5 = new AhkVar();
  send_to_workspace_6 = new AhkVar();
  send_to_workspace_7 = new AhkVar();
  send_to_workspace_8 = new AhkVar();
  send_to_workspace_9 = new AhkVar();
}
