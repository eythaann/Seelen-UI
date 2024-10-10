import { modify } from 'readable-types';
import {
  MediaWegItem,
  PinnedAppWegItem,
  PinnedWegItem,
  SeelenWegSettings,
  SeparatorWegItem,
  StartWegItem,
  SwItemType,
} from 'seelen-core';

import { IRootState } from '../../../../../shared.interfaces';
import { FocusedApp } from '../../../../shared/interfaces/common';

export type HWND = number & {};

export interface AppFromBackground {
  title: string;
  exe: string;
  execution_path: string;
  icon: string;
  icon_path: string;
  hwnd: HWND;
  creator_hwnd: HWND;
}

export enum AppsSides {
  Left = 'left',
  Center = 'center',
  Right = 'right',
  Current = 'current',
}

export interface MediaSession {
  id: string;
  title: string;
  author: string;
  thumbnail: string | null;
  playing: boolean;
  default: boolean;
  owner: {
    name: string;
    iconPath: string | null;
  } | null;
}

export type ExtendedPinnedAppWegItem = modify<
  PinnedAppWegItem,
  {
    icon: string;
    title: string;
    opens: HWND[];
  }
>;

export type ExtendedTemporalAppWegItem = modify<
  ExtendedPinnedAppWegItem,
  {
    type: SwItemType.TemporalApp;
  }
>;

export type SwItem =
  | PinnedWegItem
  | ExtendedPinnedAppWegItem
  | ExtendedTemporalAppWegItem
  | SeparatorWegItem
  | MediaWegItem
  | StartWegItem;

export interface RootState extends IRootState<SeelenWegSettings> {
  itemsOnLeft: SwItem[];
  itemsOnCenter: SwItem[];
  itemsOnRight: SwItem[];
  openApps: Record<HWND, AppFromBackground>;
  // ----------------------
  focusedApp: FocusedApp | null;
  isOverlaped: boolean;
  mediaSessions: MediaSession[];
}
