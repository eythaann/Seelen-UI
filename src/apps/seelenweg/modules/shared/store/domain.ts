import { Theme } from '../../../../../shared.interfaces';

import { SeelenWegState } from '../../../../settings/modules/seelenweg/domain';

export type HWND = number & {};

export enum SpecialItemType {
  Start = 'Start',
  Separator = 'Separator',
  Garbage = 'Garbage',
  Notification = 'Notification',
  Network = 'Network',
  Bluetooth = 'Bluetooth',
  Battery = 'Battery',
  DateTime = 'DateTime',

  PinnedApp = 'PinnedApp',
  TemporalPin = 'TemporalPin',
}

export interface UWP {
  Name: string;
  Version: string;
  PublisherId: string;
  AppId: string;
  Executable: string;
  Logo: string;
  PackageFullName: string;
  InstallLocation: string;
}

export interface App {
  type?: SpecialItemType;
  exe: string;
  icon: string;
  title: string;
  execution_path: string;
}

export interface AppFromBackground extends App {
  hwnd: HWND;
  process_hwnd: HWND;
}

export interface PinnedApp extends App {
  type: SpecialItemType.PinnedApp | SpecialItemType.TemporalPin;
  opens: HWND[];
}

export interface TemporalPinnedApp extends PinnedApp {
  type: SpecialItemType.TemporalPin;
}

export interface Separator {
  type: SpecialItemType.Separator;
}

export enum PinnedAppSide {
  LEFT = 'left',
  CENTER = 'center',
  RIGHT = 'right',
}

export interface RootState {
  pinnedOnLeft: PinnedApp[];
  pinnedOnCenter: PinnedApp[];
  pinnedOnRight: PinnedApp[];
  openApps: Record<HWND, AppFromBackground>;
  theme: Theme;
  settings: SeelenWegState;
}