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

  PinnedApp = 'PinnedApp',
  TemporalPin = 'TemporalPin',
}

export interface App {
  type?: SpecialItemType;
  exe: string;
  icon: string;
  title: string;
}

export interface AppFromBackground extends App {
  hwnd: number;
}

export interface PinnedAppSubItem {
  hwnd: number;
  title: string;
}

export interface PinnedApp extends App {
  type: SpecialItemType.PinnedApp | SpecialItemType.TemporalPin;
  opens: PinnedAppSubItem[];
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
  theme: Theme;
  settings: SeelenWegState;
}