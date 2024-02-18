import { Rect } from '../../shared/app/Rect';

import { Layout } from '../layouts/domain';

export interface Workspace {
  layout: Layout;
  name: string;
  workspacePadding: number | null;
  containerPadding: number | null;
  customLayout: any | null;
  customLayoutRules: any | null;
  layoutRules: Record<string, Layout> | null;
}

export interface Monitor {
  workAreaOffset: Rect | null;
  workspaces: Workspace[];
  edditingWorkspace: number;
}