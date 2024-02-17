import { Rect } from '../../general/main/domain';
import { Layout } from '../layouts/domain';

export interface Workspace {
  layout: Layout;
  name: string;
  workspacePadding: number | null;
  containerPadding: number | null;
  custom_layout: any | null;
  custom_layout_rules: any | null;
  layout_rules: Record<string, Layout> | null;
}

export interface Monitor {
  workAreaOffset: Rect | null;
  workspaces: Workspace[];
  edditingWorkspace: number;
}