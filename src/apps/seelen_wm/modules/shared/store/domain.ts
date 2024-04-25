import { IRootState } from '../../../../../shared.interfaces';
import { Layout } from '../../../../utils/schemas/Layout';
import { WindowManager } from '../../../../utils/schemas/WindowManager';
import { SoftOpaque } from 'readable-types/dist';

import { Reservation } from '../../layout/domain';

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
  defaultLayout: Layout;
  workspaces: Record<DesktopId, Workspace>;
  activeWorkspace: DesktopId;
  /** current focused window handle */
  activeWindow: number;
  /** last managed window activated */
  lastManagedActivated: number | null;
  reservation: Reservation | null;
  desktopByHandle: Record<number, DesktopId>;
  /** Prop to listen for app forced updates */
  version: number;
}