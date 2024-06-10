import { CreatorInfoSchema } from '.';
import z from 'zod';

export enum ToolbarModuleType {
  Generic = 'generic',
  Text = 'text',
  Date = 'date',
  Power = 'power',
  Settings = 'settings',
  Network = 'network',
  Workspaces = 'workspaces',
  Sound = 'sound',
  Bluetooth = 'bluetooth',
  Tray = 'tray',
}

export enum WorkspaceTMMode {
  Dotted = 'dotted',
  Named = 'named',
  Numbered = 'numbered',
}

export enum TimeUnit {
  SECOND = 'second',
  MINUTE = 'minute',
  HOUR = 'hour',
  DAY = 'day',
}

export const BaseTMSchema = z.object({
  type: z.nativeEnum(ToolbarModuleType),
  template: z
    .string()
    .transform((value) => value.trimEnd())
    .refine((value) => !value.endsWith('\n'), {
      message: 'Template must not end with a newline',
    })
    .default('"Unset"'),
  tooltip: z.string().nullable().default(null),
  onClick: z.string().nullable().default(null),
});

export type GenericToolbarModule = z.infer<typeof GenericToolbarModuleSchema>;
export const GenericToolbarModuleSchema = BaseTMSchema.extend({
  type: z.union([z.literal(ToolbarModuleType.Generic), z.literal(ToolbarModuleType.Text)]),
});

export type TrayTM = z.infer<typeof TrayTMSchema>;
export const TrayTMSchema = BaseTMSchema.extend({
  type: z.literal(ToolbarModuleType.Tray),
});

export type DateToolbarModule = z.infer<typeof DateToolbarModuleSchema>;
export const DateToolbarModuleSchema = BaseTMSchema.extend({
  type: z.literal(ToolbarModuleType.Date),
  each: z
    .nativeEnum(TimeUnit)
    .describe('Time unit to update the showing date')
    .default(TimeUnit.MINUTE),
  format: z.string().default('MMM Do, HH:mm'),
});

export type PowerToolbarModule = z.infer<typeof PowerToolbarModuleSchema>;
export const PowerToolbarModuleSchema = BaseTMSchema.extend({
  type: z.literal(ToolbarModuleType.Power),
});

export type SettingsToolbarModule = z.infer<typeof SettingsToolbarModuleSchema>;
export const SettingsToolbarModuleSchema = BaseTMSchema.extend({
  type: z.literal(ToolbarModuleType.Settings),
});

export type WorkspacesTM = z.infer<typeof WorkspaceTMSchema>;
export const WorkspaceTMSchema = BaseTMSchema.extend({
  type: z.literal(ToolbarModuleType.Workspaces),
  mode: z.nativeEnum(WorkspaceTMMode).default(WorkspaceTMMode.Numbered),
});

export type ToolbarModule = z.infer<typeof ToolbarModuleSchema>;
export const ToolbarModuleSchema = z.union([
  GenericToolbarModuleSchema,
  DateToolbarModuleSchema,
  PowerToolbarModuleSchema,
  SettingsToolbarModuleSchema,
  WorkspaceTMSchema,
  TrayTMSchema,
]);

type InnerPlaceholder = z.infer<typeof PlaceholderSchema>;
export const PlaceholderSchema = z.object({
  info: CreatorInfoSchema.default({}),
  left: z.array(ToolbarModuleSchema).default([]),
  center: z.array(ToolbarModuleSchema).default([]),
  right: z.array(ToolbarModuleSchema).default([]),
});

export interface Placeholder extends InnerPlaceholder {
  info: InnerPlaceholder['info'] & {
    filename: string;
  };
}
