import { IRootState } from '../../../../../shared.interfaces';
import { Seelenweg } from '../../../../shared/schemas/Seelenweg';
import {
  SavedMediaItem,
  SavedPinnedApp,
  SavedSeparatorItem,
  SwItemType as SpecialItemType,
} from '../../../../shared/schemas/SeelenWegItems';
import { modify } from 'readable-types';

export type HWND = number & {};

export { SpecialItemType };

export interface AppFromBackground {
  title: string;
  exe: string;
  execution_path: string;
  icon: string;
  icon_path: string;
  hwnd: HWND;
  process_hwnd: HWND;
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

export type SwItem = SwPinnedApp | SwTemporalApp | SavedSeparatorItem | SavedMediaItem;

export interface RootState extends IRootState<Seelenweg> {
  itemsOnLeft: SwItem[];
  itemsOnCenter: SwItem[];
  itemsOnRight: SwItem[];
  openApps: Record<HWND, AppFromBackground>;
  // ----------------------
  focusedHandle: HWND;
  isOverlaped: boolean;
  mediaSessions: MediaSession[];
}
