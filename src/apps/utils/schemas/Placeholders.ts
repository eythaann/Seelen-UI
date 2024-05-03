import { CreatorInfoSchema } from '.';
import z from 'zod';

export enum ToolbarModuleType {
  GENERIC = 'generic',
  TEXT = 'text',
  DATE = 'date',
  POWER = 'power',
  NETWORK = 'network',
  SOUND = 'sound',
  BLUETOOTH = 'bluetooth',
  TRAY = 'tray',
}

window.TOOLBAR_MODULES = {
  [ToolbarModuleType.GENERIC]: false,
  [ToolbarModuleType.TEXT]: false,
  [ToolbarModuleType.DATE]: false,
  [ToolbarModuleType.POWER]: false,
  [ToolbarModuleType.NETWORK]: false,
  [ToolbarModuleType.SOUND]: false,
  [ToolbarModuleType.BLUETOOTH]: false,
  [ToolbarModuleType.TRAY]: false,
};

export enum TimeUnit {
  SECOND = 'second',
  MINUTE = 'minute',
  HOUR = 'hour',
  DAY = 'day',
}

export type BaseToolbarModule = z.infer<typeof BaseToolbarModuleSchema>;
export const BaseToolbarModuleSchema = z.object({
  type: z.nativeEnum(ToolbarModuleType),
  template: z
    .string()
    .transform((value) => value.trimEnd())
    .refine((value) => !value.endsWith('\n'), {
      message: 'Template must not end with a newline',
    }),
  tooltip: z.string().nullable().default(null),
  onClick: z.string().nullable().default(null),
});

export type GenericToolbarModule = z.infer<typeof GenericToolbarModuleSchema>;
export const GenericToolbarModuleSchema = BaseToolbarModuleSchema.extend({
  type: z.union([z.literal(ToolbarModuleType.GENERIC), z.literal(ToolbarModuleType.TEXT)]),
});

export type DateToolbarModule = z.infer<typeof DateToolbarModuleSchema>;
export const DateToolbarModuleSchema = BaseToolbarModuleSchema.extend({
  type: z.literal(ToolbarModuleType.DATE),
  each: z
    .nativeEnum(TimeUnit)
    .describe('Time unit to update the showing date')
    .default(TimeUnit.MINUTE),
  format: z.string().default('MMM Do, HH:mm'),
});

export type PowerToolbarModule = z.infer<typeof PowerToolbarModuleSchema>;
export const PowerToolbarModuleSchema = BaseToolbarModuleSchema.extend({
  type: z.literal(ToolbarModuleType.POWER),
});

export type ToolbarModule = z.infer<typeof ToolbarModuleSchema>;
export const ToolbarModuleSchema = z.union([
  GenericToolbarModuleSchema,
  DateToolbarModuleSchema,
  PowerToolbarModuleSchema,
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
