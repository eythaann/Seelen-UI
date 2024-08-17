import { RectSchema } from './WindowManager';
import z from 'zod';

export type Workspace = z.infer<typeof WorkspaceSchema>;
export const WorkspaceSchema = z.object({
  name: z.string().default('New Workspace'),
  layout: z.string().default('BSP'),
  padding: z.number().nonnegative().optional().nullable(),
  gap: z.number().nonnegative().optional().nullable(),
});

type InnerMonitor = z.infer<typeof MonitorSchema>;
export const MonitorSchema = z.object({
  workspaces: z.array(WorkspaceSchema).min(1).default([WorkspaceSchema.parse({})]),
  work_area_offset: RectSchema.optional().nullable(),
  editing_workspace: z.number().nonnegative().default(0),
});

export interface Monitor {
  workAreaOffset: InnerMonitor['work_area_offset'];
  workspaces: InnerMonitor['workspaces'];
  edditingWorkspace: InnerMonitor['editing_workspace'];
}