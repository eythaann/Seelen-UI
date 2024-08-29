import { IRootState } from '../../../../../shared.interfaces';
import { FocusedApp } from '../../../../shared/interfaces/common';
import { Seelenweg } from '../../../../shared/schemas/Seelenweg';
import {
  SavedMediaItem,
  SavedPinnedApp,
  SavedSeparatorItem,
  StartMenuItem,
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

export interface UIColors {
  background: string;
  foreground: string;
  accent_darkest: string;
  accent_darker: string;
  accent_dark: string;
  accent: string;
  accent_light: string;
  accent_lighter: string;
  accent_lightest: string;
  complement: string | null;
}

export interface RootState extends IRootState<Seelenweg> {
  itemsOnLeft: SwItem[];
  itemsOnCenter: SwItem[];
  itemsOnRight: SwItem[];
  openApps: Record<HWND, AppFromBackground>;
  // ----------------------
  focusedApp: FocusedApp | null;
  isOverlaped: boolean;
  mediaSessions: MediaSession[];
  colors: UIColors;
}
