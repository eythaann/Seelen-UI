import { Rect } from '../../shared/app/Rect';

import { Layout } from '../layouts/domain';

export interface Workspace {
  layout: Layout;
  name: string;
  workspacePadding: number | null;
  containerPadding: number | null;
  customLayout: string | null;
  customLayoutRules: Record<string, string | null> | null;
  layoutRules: Record<string, Layout | null> | null;
}

export class Workspace {
  static default(): Workspace {
    return {
      name: 'Workspace 1',
      layout: Layout.BSP,
      layoutRules: null,
      containerPadding: null,
      workspacePadding: null,
      customLayout: null,
      customLayoutRules: null,
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