
import { WindowManagerSettings, WmNode } from '@seelen-ui/lib/types';

import { Reservation } from '../../layout/domain';

import { IRootState } from '../../../../../shared.interfaces';

export enum FocusAction {
  Left = 'Left',
  Right = 'Right',
  Up = 'Up',
  Down = 'Down',
  Latest = 'Latest',
}

export interface RootState extends IRootState<WindowManagerSettings> {
  _version: number;
  layout: WmNode | null;
  activeWindow: number;
  reservation: Reservation | null;
  overlayVisible: boolean;
}
