import { IRootState } from '../../../../../shared.interfaces';
import { Layout } from '../../../../shared/schemas/Layout';
import { WindowManager } from '../../../../shared/schemas/WindowManager';
import { SoftOpaque } from 'readable-types';

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
  Latest = 'Latest',
}

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

export interface RootState extends IRootState<WindowManager> {
  colors: UIColors;
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

export interface AddWindowPayload {
  hwnd: HWND;
  desktop_id: DesktopId;
  as_floating: boolean;
}
