import { Rect } from '../../shared/app/Rect';

import { Layout } from '../layouts/domain';

export interface Workspace {
  name: string;
  layout: Layout;
  workspacePadding: number | null;
  containerPadding: number | null;
}

export class Workspace {
  static default(): Workspace {
    return {
      name: 'Workspace 1',
      layout: Layout.BSP,
      containerPadding: null,
      workspacePadding: null,
    };
  }
}

export interface Monitor {
  workAreaOffset: Rect | null;
  workspaces: Workspace[];
  edditingWorkspace: number;
}

export class Monitor {
  static default(): Monitor {
    return {
      edditingWorkspace: 0,
      workAreaOffset: null,
      workspaces: [Workspace.default()],
    };
  }
}