import { SoftOpaque } from 'readable-types/dist';

import { Layout, Reservation } from '../../layout/domain';

interface FloatingWindowSettings {
  width: number;
  height: number;
}

interface WMSettings {
  floating: FloatingWindowSettings;
}

interface Workspace {
  name: string;
  layout: Layout;
}

export type DesktopId = SoftOpaque<string, 'DesktopId'>;

export interface RootState {
  settings: WMSettings;
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