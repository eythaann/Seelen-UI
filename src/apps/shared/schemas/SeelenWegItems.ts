import { z } from 'zod';

export enum SwItemType {
  PinnedApp = 'PinnedApp',
  TemporalApp = 'TemporalPin',
  Separator = 'Separator',
  Media = 'Media',
}

export type SavedPinnedApp = z.infer<typeof PinnedAppSchema>;
const PinnedAppSchema = z.object({
  type: z.literal(SwItemType.PinnedApp),
  icon_path: z.string(),
  /** Path to executable */
  exe: z.string(),
  /** Path to execute the app using explorer.exe (uwp apps starts with `shell:AppsFolder`) */
  execution_path: z.string(),
});

export type SavedSeparatorItem = z.infer<typeof SeparatorSchema>;
const SeparatorSchema = z.object({
  type: z.literal(SwItemType.Separator),
});

export type SavedMediaItem = z.infer<typeof MediaItemSchema>;
const MediaItemSchema = z.object({
  type: z.literal(SwItemType.Media),
});

export type SwSavedItem = z.infer<typeof SwSavedItemSchema>;
export const SwSavedItemSchema = z.union([
  PinnedAppSchema,
  SeparatorSchema,
  MediaItemSchema,
]);

export type SwSaveFile = z.infer<typeof SwSaveFileSchema>;
export const SwSaveFileSchema = z.object({
  left: z.array(SwSavedItemSchema),
  center: z.array(SwSavedItemSchema),
  right: z.array(SwSavedItemSchema),
});