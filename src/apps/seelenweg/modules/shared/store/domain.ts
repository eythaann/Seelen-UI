import { SeelenWegSettings, WegItem } from '@seelen-ui/lib/types';

import { AppNotification } from 'src/apps/toolbar/modules/Notifications/domain';

import { IRootState } from '../../../../../shared.interfaces';
import { FocusedApp } from '../../../../shared/interfaces/common';

export type HWND = number & {};

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
export type TemporalWegItem = Extract<WegItem, { type: 'Temporal' }>;
export type SeparatorWegItem = Extract<WegItem, { type: 'Separator' }>;
export type MediaWegItem = Extract<WegItem, { type: 'Media' }>;
export type StartMenuWegItem = Extract<WegItem, { type: 'StartMenu' }>;

/** @alias */
export type SwItem = WegItem;

export interface RootState extends IRootState<SeelenWegSettings> {
  devTools: boolean;
  itemsOnLeft: SwItem[];
  itemsOnCenter: SwItem[];
  itemsOnRight: SwItem[];
  // ----------------------
  focusedApp: FocusedApp | null;
  isOverlaped: boolean;
  mediaSessions: MediaSession[];
  notifications: AppNotification[];
}
