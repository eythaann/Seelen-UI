import { Theme } from '../../../../../shared.interfaces';

import { SeelenWegState } from '../../../../settings/modules/seelenweg/domain';

export enum SpecialItemType {
  Start = 'Start',
  Separator = 'Separator',
  Garbage = 'Garbage',
  Notification = 'Notification',
  Network = 'Network',
  Bluetooth = 'Bluetooth',
  Battery = 'Battery',
  DateTime = 'DateTime',
  App = 'App',
}

export interface App {
  exe: string;
  icon: string;
  title: string;
}

export interface SpecialApp {
  type: SpecialItemType;
}

export interface OpenApp extends App {
  hwnd: number;
}

export interface PinnedApp extends App, SpecialApp {
  hwnd?: never;
}

export enum PinnedAppSide {
  LEFT = 'left',
  CENTER = 'center',
  RIGHT = 'right',
}

export interface RootState {
  apps: OpenApp[];
  pinnedOnLeft: PinnedApp[];
  pinnedOnCenter: PinnedApp[];
  pinnedOnRight: PinnedApp[];
  theme: Theme;
  settings: SeelenWegState;
}