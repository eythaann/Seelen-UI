import { Rect } from '../utils';
import { SeelenWallWallpaper } from './settings';

export class FancyToolbarSettingsByMonitor {
  enabled: boolean = true;
}

export class SeelenWegSettingsByMonitor {
  enabled: boolean = true;
}

export class WindowManagerSettingsByMonitor {
  enabled: boolean = true;
  padding: number | null = null;
  margin: Rect | null = null;
  gap: number | null = null;
  layout: string | null = null;
}

export class SeelenWallSettingsByMonitor {
  enabled: boolean = true;
  backgrounds: SeelenWallWallpaper[] | null = null;
}

export enum WorkspaceIdentifierType {
  Name = 'name',
  Index = 'index',
}

export class WorkspaceIdentifier {
  id: string;
  kind: WorkspaceIdentifierType;

  constructor(id: string, kind: WorkspaceIdentifierType) {
    this.id = id;
    this.kind = kind;
  }
}

export class WorkspaceConfiguration {
  identifier: WorkspaceIdentifier;
  layout: string | null = null;
  backgrounds: SeelenWallWallpaper[] | null = null;

  constructor(identifier: WorkspaceIdentifier) {
    this.identifier = identifier;
  }
}

export class MonitorConfiguration {
  tb: FancyToolbarSettingsByMonitor = new FancyToolbarSettingsByMonitor();
  wall: SeelenWallSettingsByMonitor = new SeelenWallSettingsByMonitor();
  weg: SeelenWegSettingsByMonitor = new SeelenWegSettingsByMonitor();
  wm: WindowManagerSettingsByMonitor = new WindowManagerSettingsByMonitor();
  /** list of settings by workspace on this monitor */
  workspaces: WorkspaceConfiguration[] = [];
}
