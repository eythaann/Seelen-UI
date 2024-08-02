import { z } from 'zod';

export enum SwItemType {
  PinnedApp = 'PinnedApp',
  TemporalApp = 'TemporalPin',
  Separator = 'Separator',
  Media = 'Media',
  Start = 'StartMenu',
}

export type SavedPinnedApp = z.infer<typeof PinnedAppSchema>;
const PinnedAppSchema = z.object({
  type: z.literal(SwItemType.PinnedApp),
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

export type StartMenuItem = z.infer<typeof StartMenuItemSchema>;
const StartMenuItemSchema = z.object({
  type: z.literal(SwItemType.Start),
});

export type SwSavedItem = z.infer<typeof SwSavedItemSchema>;
export const SwSavedItemSchema = z.union([
  PinnedAppSchema,
  SeparatorSchema,
  MediaItemSchema,
  StartMenuItemSchema,
]);

export type SwSaveFile = z.infer<typeof SwSaveFileSchema>;
export const SwSaveFileSchema = z.object({
  left: z.array(SwSavedItemSchema).default([
    {
      type: SwItemType.Start,
    },
  ]),
  center: z.array(SwSavedItemSchema).default([]),
  right: z.array(SwSavedItemSchema).default([
    {
      type: SwItemType.Media,
    },
  ]),
});
