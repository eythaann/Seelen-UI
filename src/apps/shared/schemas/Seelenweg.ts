import z from 'zod';

export enum SeelenWegMode {
  FULL_WIDTH = 'Full-Width',
  MIN_CONTENT = 'Min-Content',
}

export enum AppBarHideMode {
  Never = 'Never',
  Always = 'Always',
  OnOverlap = 'On-Overlap',
}

export enum SeelenWegSide {
  LEFT = 'Left',
  RIGHT = 'Right',
  TOP = 'Top',
  BOTTOM = 'Bottom',
}

export const SeelenWegSchema = z.object({
  enabled: z.boolean().default(true),
  mode: z.nativeEnum(SeelenWegMode).default(SeelenWegMode.MIN_CONTENT),
  hide_mode: z.nativeEnum(AppBarHideMode).default(AppBarHideMode.OnOverlap),
  position: z.nativeEnum(SeelenWegSide).default(SeelenWegSide.BOTTOM),
  visible_separators: z.boolean().default(true),
  size: z.number().positive().default(40).describe('Item size in pixels'),
  zoom_size: z.number().positive().default(70).describe('Zoomed item size in pixels'),
  margin: z.number().nonnegative().default(8).describe('Dock/Bar margin in pixels'),
  padding: z.number().nonnegative().default(8).describe('Dock/Bar padding in pixels'),
  space_between_items: z.number().nonnegative().default(8).describe('Space between items (gap) in pixels'),
});

type inner = z.infer<typeof SeelenWegSchema> & {};
export interface Seelenweg {
  enabled: inner['enabled'];
  mode: inner['mode'];
  hideMode: inner['hide_mode'];
  position: inner['position'];
  visibleSeparators: inner['visible_separators'];
  size: inner['size'];
  zoomSize: inner['zoom_size'];
  margin: inner['margin'];
  padding: inner['padding'];
  spaceBetweenItems: inner['space_between_items'];
}