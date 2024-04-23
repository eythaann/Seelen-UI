import { IRootState } from '../../../../../shared.interfaces';
import { SoftOpaque } from 'readable-types/dist';

import { SeelenManagerState } from '../../../../settings/modules/WindowManager/main/domain';
import { Layout, Reservation } from '../../layout/domain';

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

export interface RootState extends IRootState<SeelenManagerState> {
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