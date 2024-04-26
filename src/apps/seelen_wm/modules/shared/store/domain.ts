import { IRootState } from '../../../../../shared.interfaces';
import { Layout } from '../../../../utils/schemas/Layout';
import { WindowManager } from '../../../../utils/schemas/WindowManager';
import { SoftOpaque } from 'readable-types/dist';

import { Reservation } from '../../layout/domain';
import { HWND } from '../utils/domain';

interface Workspace {
  name: string;
  layout: Layout;
}

export type DesktopId = SoftOpaque<string, 'DesktopId'>;

export enum FocusAction {
  Left = 'Left',
  Right = 'Right',
  Up = 'Up',
  Down = 'Down',
  Lastest = 'Lastest',
}

export interface RootState extends IRootState<WindowManager> {
  availableLayouts: Layout[];
  workspaces: Record<DesktopId, Workspace>;
  activeWorkspace: DesktopId;
  /** current focused window handle */
  activeWindow: HWND;
  /** last managed window activated */
  lastManagedActivated: HWND | null;
  reservation: Reservation | null;
  handlesByDesktop: Record<DesktopId, HWND[]>;
  desktopByHandle: Record<HWND, DesktopId>;
  /** Prop to listen for app forced updates */
  version: number;
}