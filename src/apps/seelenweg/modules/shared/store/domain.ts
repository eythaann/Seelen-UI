import { IRootState } from '../../../../../shared.interfaces';
import { FocusedApp } from '../../../../shared/interfaces/common';
import {
  SavedMediaItem,
  SavedPinnedApp,
  SavedSeparatorItem,
  StartMenuItem,
  SwItemType as SpecialItemType,
} from '../../../../shared/schemas/SeelenWegItems';
import { modify } from 'readable-types';
import { SeelenWegSettings } from 'seelen-core';

export type HWND = number & {};

export { SpecialItemType };

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

export type SwPinnedApp = modify<
  SavedPinnedApp,
  {
    icon: string;
    title: string;
    opens: HWND[];
  }
>;

export type SwTemporalApp = modify<
  SwPinnedApp,
  {
    type: SpecialItemType.TemporalApp;
  }
>;

export type SwItem = SwPinnedApp | SwTemporalApp | SavedSeparatorItem | SavedMediaItem | StartMenuItem;

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
