import z from 'zod';

export enum ContainerTopBarMode {
  ON_STACK = 'OnStack',
  NEVER = 'Never',
}

export type Rect = z.infer<typeof RectSchema>;
export const RectSchema = z.object({
  top: z.number().default(0),
  left: z.number().default(0),
  right: z.number().default(0),
  bottom: z.number().default(0),
});

export type Border = z.infer<typeof BorderSchema>;
export const BorderSchema = z.object({
  enabled: z.boolean().default(true),
  width: z.number().min(0).default(3),
  offset: z.number().default(-1),
});

export type ContainerTabs = z.infer<typeof ContainerTabsSchema>;
export const ContainerTabsSchema = z.object({
  mode: z.nativeEnum(ContainerTopBarMode).default(ContainerTopBarMode.ON_STACK),
});

export type FloatingWindowSettings = z.infer<typeof FloatingWindowSchema>;
export const FloatingWindowSchema = z.object({
  width: z.number().positive().default(800),
  height: z.number().positive().default(500),
});

export const WindowManagerSchema = z.object({
  enabled: z.boolean().default(true),
  auto_stackin_by_category: z.boolean().default(true),
  border: BorderSchema.default({}),
  resize_delta: z.number().default(10).describe('% to add or remove on resize of windows using the CLI'),
  workspace_gap: z.number().nonnegative().default(10).describe('Space between windows'),
  workspace_padding: z.number().nonnegative().default(10),
  global_work_area_offset: RectSchema.default({}),
  container_top_bar: ContainerTabsSchema.default({}),
  floating: FloatingWindowSchema.default({}),
});

type inner = z.infer<typeof WindowManagerSchema> & {};
export interface WindowManager {
  enabled: inner['enabled'];
  autoStackinByCategory: inner['auto_stackin_by_category'];
  border: inner['border'];
  resizeDelta: inner['resize_delta'];
  workspaceGap: inner['workspace_gap'];
  workspacePadding: inner['workspace_padding'];
  globalWorkAreaOffset: inner['global_work_area_offset'];
  containerTopBar: inner['container_top_bar'];
  floating: inner['floating'];
}