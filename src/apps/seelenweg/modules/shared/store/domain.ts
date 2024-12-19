import { SeelenWegSettings, WegItem } from '@seelen-ui/lib/types';
import { modify } from 'readable-types';

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

export type PinnedWegItem = Extract<WegItem, { type: 'Pinned' }>;
export type ExtendedPinnedWegItem = modify<
  PinnedWegItem,
  {
    icon: string;
    title: string;
    opens: HWND[];
  }
>;

export type ExtendedTemporalWegItem = modify<
  ExtendedPinnedWegItem,
  {
    type: 'Temporal';
  }
>;

export type SeparatorWegItem = Extract<WegItem, { type: 'Separator' }>;
export type MediaWegItem = Extract<WegItem, { type: 'Media' }>;
export type StartMenuWegItem = Extract<WegItem, { type: 'StartMenu' }>;
export type SwItem =
  | ExtendedPinnedWegItem
  | ExtendedTemporalWegItem
  | SeparatorWegItem
  | MediaWegItem
  | StartMenuWegItem;

export interface RootState extends IRootState<SeelenWegSettings> {
  devTools: boolean;
  itemsOnLeft: SwItem[];
  itemsOnCenter: SwItem[];
  itemsOnRight: SwItem[];
  openApps: Record<HWND, AppFromBackground>;
  // ----------------------
  focusedApp: FocusedApp | null;
  isOverlaped: boolean;
  mediaSessions: MediaSession[];
}
