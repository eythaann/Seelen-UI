import { IRootState } from '../../../../../shared.interfaces';
import { Seelenweg } from '../../../../utils/schemas/Seelenweg';
import { modify } from 'readable-types';

import { PinnedApp } from '../../item/app/PinnedApp';
import { TemporalApp } from '../../item/app/TemporalApp';

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

/** @from pwsh */
export interface UWP_App {
  AppId: string;
  /** Relative path to the executable from Package:IntallLocation folder */
  Executable: string;
  /** An image used as the app's Start Screen medium tile, and on the Task Switcher. */
  Square150x150Logo: string;
  /** An image used as the app's Start Screen small tile, and on the All Apps List (taskbar). */
  Square44x44Logo: string;
}

/** @from pwsh */
export interface UWP_Package {
  Name: string;
  Version: string;
  PublisherId: string;
  PackageFullName: string;
  InstallLocation: string;
  StoreLogo: string;
  Applications: UWP_App[];
}

export interface IApp {
  type?: SpecialItemType;
  /** Path to executable */
  exe: string;
  /** Base64 image or URL */
  icon: string;
  icon_path: string;
  title: string;
  /** Path to execute the app using explorer.exe (uwp apps starts with `shell:AppsFolder`) */
  execution_path: string;
}

export type AppFromBackground = modify<IApp, {
  /** Base64 image or URL */
  icon: string | null;
  hwnd: HWND;
  process_hwnd: HWND;
}>;

export type SavedAppsInYaml = Omit<IApp, 'icon'>;

export interface Separator {
  type: SpecialItemType.Separator;
}

export type App = PinnedApp | TemporalApp;

export enum AppsSides {
  LEFT = 'left',
  CENTER = 'center',
  RIGHT = 'right',
}

export interface RootState extends IRootState<Seelenweg> {
  pinnedOnLeft: App[];
  pinnedOnCenter: App[];
  pinnedOnRight: App[];
  openApps: Record<HWND, AppFromBackground>;
  focusedHandle: HWND;
  isOverlaped: boolean;
}