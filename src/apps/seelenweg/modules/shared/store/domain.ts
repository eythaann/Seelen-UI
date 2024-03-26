import { Theme } from '../../../../../shared.interfaces';

import { SeelenWegState } from '../../../../settings/modules/seelenweg/domain';

export interface App {
  exe: string;
  icon: string;
}

export interface OpenApp extends App {
  state: 'Open';
  title: string;
  hwnd: number;
}

export interface PinnedApp extends App {
  state: 'Pinned';
  hwnd: 0;
}

export interface RootState {
  apps: OpenApp[];
  pinnedOnLeft: PinnedApp[];
  pinnedOnCenter: PinnedApp[];
  pinnedOnRight: PinnedApp[];
  theme: Theme | null;
  settings: SeelenWegState;
}