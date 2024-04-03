import { Theme } from '../../../../../shared.interfaces';
import { modify } from 'readable-types';

import { PinnedApp } from '../../item/app/PinnedApp';
import { TemporalApp } from '../../item/app/TemporalApp';

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

export interface IApp {
  type?: SpecialItemType;
  exe: string;
  /** Base64 image or URL */
  icon: string;
  icon_path: string;
  title: string;
  execution_path: string;
}

export type AppFromBackground = modify<IApp, {
  icon: string | null;
  hwnd: HWND;
  process_hwnd: HWND;
}>;

export type SavedItems = modify<IApp, { icon: string | null }>;

export interface Separator {
  type: SpecialItemType.Separator;
}

export type App = PinnedApp | TemporalApp;

export enum AppsSides {
  LEFT = 'left',
  CENTER = 'center',
  RIGHT = 'right',
}

export interface RootState {
  pinnedOnLeft: App[];
  pinnedOnCenter: App[];
  pinnedOnRight: App[];
  openApps: Record<HWND, AppFromBackground>;
  theme: Theme;
  settings: SeelenWegState;
}