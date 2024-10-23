import { Obtainable, SeelenCommand, SeelenEvent } from '../handlers';
import { Rect } from '../utils';
import { MonitorConfiguration } from './settings_by_monitor';

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
  NeverWithoutPlaceholder = 'Never without placeholder',
  Always = 'Always',
  OnOverlap = 'On-Overlap',
}

export enum SeelenWegSide {
  Left = 'Left',
  Right = 'Right',
  Top = 'Top',
  Bottom = 'Bottom',
}

export class SeelenWallWallpaper {
  id: string = crypto.randomUUID();
  path: string = '';
}

export class SeelenWallSettings {
  enabled: boolean = true;
  backgrounds: SeelenWallWallpaper[] = [];
  /** Interval in seconds */
  interval: number = 60;
}

export enum SeelenLauncherMonitor {
  Primary = 'Primary',
  MouseOver = 'Mouse-Over',
}

export class SeelenLauncherRunner {
  id: string = crypto.randomUUID();
  label: string = '';
  program: string = '';
  readonly: boolean = false;
}

export class SeelenLauncherSettings {
  enabled: boolean = false;
  monitor: SeelenLauncherMonitor = SeelenLauncherMonitor.MouseOver;
  runners: SeelenLauncherRunner[] = [];
}

export enum UpdateChannel {
  Release = 'Release',
  Beta = 'Beta',
  Nightly = 'Nightly',
}

export class UpdaterSettings {
  channel: UpdateChannel = UpdateChannel.Nightly;
}

export class Settings extends Obtainable<Settings>(
  SeelenCommand.StateGetSettings,
  SeelenEvent.StateSettingsChanged,
) {
  fancyToolbar: FancyToolbarSettings = new FancyToolbarSettings();
  seelenweg: SeelenWegSettings = new SeelenWegSettings();
  windowManager: WindowManagerSettings = new WindowManagerSettings();
  wall: SeelenWallSettings = new SeelenWallSettings();
  launcher: SeelenLauncherSettings = new SeelenLauncherSettings();
  monitors: MonitorConfiguration[] = [new MonitorConfiguration()];
  ahkEnabled: boolean = true;
  ahkVariables: AhkVarList = new AhkVarList();
  selectedThemes: string[] = ['default'];
  iconPacks: string[] = ['system'];
  devTools: boolean = false;
  language: string = '';
  dateFormat: string = 'ddd D MMM, hh:mm A';
  virtualDesktopStrategy: VirtualDesktopStrategy = VirtualDesktopStrategy.Native;
  updater: UpdaterSettings = new UpdaterSettings();
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
  workspaceMargin: Rect = new Rect();
  floating: FloatingWindowSettings = new FloatingWindowSettings();
  defaultLayout: string = 'default.yml';
}

export class AhkVar {
  fancy: string;
  ahk: string;
  readonly: boolean = false;

  constructor(fancy: string = '', ahk: string = '') {
    this.fancy = fancy;
    this.ahk = ahk;
  }
}

/// TODO: find the way to avoid duplicated code between rust and this class
export class AhkVarList {
  // launcher
  toggleLauncher = new AhkVar('Win + Space', 'LWin & Space');
  // wm
  reserveTop = new AhkVar('Win + Shift + I', '#+i');
  reserveBottom = new AhkVar('Win + Shift + K', '#+k');
  reserveLeft = new AhkVar('Win + Shift + J', '#+j');
  reserveRight = new AhkVar('Win + Shift + L', '#+l');
  reserveFloat = new AhkVar('Win + Shift + U', '#+u');
  reserveStack = new AhkVar('Win + Shift + O', '#+o');
  focusTop = new AhkVar('Win + Shift + W', '#+w');
  focusBottom = new AhkVar('Win + Shift + S', '#+s');
  focusLeft = new AhkVar('Win + Shift + A', '#+a');
  focusRight = new AhkVar('Win + Shift + D', '#+d');
  focusLatest = new AhkVar('Win + Shift + E', '#+e');
  increaseWidth = new AhkVar('Win + Alt + =', '#!=');
  decreaseWidth = new AhkVar('Win + Alt + -', '#!-');
  increaseHeight = new AhkVar('Win + Shift + =', '#+=');
  decreaseHeight = new AhkVar('Win + Shift + -', '#+-');
  restoreSizes = new AhkVar('Win + Alt + 0', '#!0');
  // virtual desktops
  switchWorkspace0 = new AhkVar('Alt + 1', '!1');
  switchWorkspace1 = new AhkVar('Alt + 2', '!2');
  switchWorkspace2 = new AhkVar('Alt + 3', '!3');
  switchWorkspace3 = new AhkVar('Alt + 4', '!4');
  switchWorkspace4 = new AhkVar('Alt + 5', '!5');
  switchWorkspace5 = new AhkVar('Alt + 6', '!6');
  switchWorkspace6 = new AhkVar('Alt + 7', '!7');
  switchWorkspace7 = new AhkVar('Alt + 8', '!8');
  switchWorkspace8 = new AhkVar('Alt + 9', '!9');
  switchWorkspace9 = new AhkVar('Alt + 0', '!0');
  moveToWorkspace0 = new AhkVar('Alt + Shift + 1', '!+1');
  moveToWorkspace1 = new AhkVar('Alt + Shift + 2', '!+2');
  moveToWorkspace2 = new AhkVar('Alt + Shift + 3', '!+3');
  moveToWorkspace3 = new AhkVar('Alt + Shift + 4', '!+4');
  moveToWorkspace4 = new AhkVar('Alt + Shift + 5', '!+5');
  moveToWorkspace5 = new AhkVar('Alt + Shift + 6', '!+6');
  moveToWorkspace6 = new AhkVar('Alt + Shift + 7', '!+7');
  moveToWorkspace7 = new AhkVar('Alt + Shift + 8', '!+8');
  moveToWorkspace8 = new AhkVar('Alt + Shift + 9', '!+9');
  moveToWorkspace9 = new AhkVar('Alt + Shift + 0', '!+0');
  sendToWorkspace0 = new AhkVar('Win + Shift + 1', '#+1');
  sendToWorkspace1 = new AhkVar('Win + Shift + 2', '#+2');
  sendToWorkspace2 = new AhkVar('Win + Shift + 3', '#+3');
  sendToWorkspace3 = new AhkVar('Win + Shift + 4', '#+4');
  sendToWorkspace4 = new AhkVar('Win + Shift + 5', '#+5');
  sendToWorkspace5 = new AhkVar('Win + Shift + 6', '#+6');
  sendToWorkspace6 = new AhkVar('Win + Shift + 7', '#+7');
  sendToWorkspace7 = new AhkVar('Win + Shift + 8', '#+8');
  sendToWorkspace8 = new AhkVar('Win + Shift + 9', '#+9');
  sendToWorkspace9 = new AhkVar('Win + Shift + 0', '#+0');
  // miscellaneous
  miscOpenSettings = new AhkVar('Win + K', '#k');
  miscToggleLockTracing = new AhkVar('Ctrl + Win + Alt + T', '^#!t');
  miscToggleWinEventTracing = new AhkVar('Ctrl + Win + Alt + L', '^#!l');
}
