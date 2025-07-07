import { AppNotification, FocusedApp, MediaPlayer, WegItem } from '@seelen-ui/lib/types';

export type HWND = number & {};

export type PinnedWegItem = Extract<WegItem, { type: 'Pinned' }>;
export type TemporalWegItem = Extract<WegItem, { type: 'Temporal' }>;
export type SeparatorWegItem = Extract<WegItem, { type: 'Separator' }>;
export type MediaWegItem = Extract<WegItem, { type: 'Media' }>;
export type StartMenuWegItem = Extract<WegItem, { type: 'StartMenu' }>;

/** @alias */
export type SwItem = WegItem;

export interface RootState {
  devTools: boolean;
  itemsOnLeft: SwItem[];
  itemsOnCenter: SwItem[];
  itemsOnRight: SwItem[];
  reorderDisabled: boolean;
  // ----------------------
  focusedApp: FocusedApp | null;
  mediaSessions: MediaPlayer[];
  notifications: AppNotification[];
}
