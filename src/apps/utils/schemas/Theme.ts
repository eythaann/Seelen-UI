import { CreatorInfoSchema } from '.';
import { modify } from 'readable-types/dist';
import { z } from 'zod';

const backgroundLayersSchema = z.number().min(0).default(0);

type inner = z.infer<typeof ThemeSchema>;
export const ThemeSchema = z.object({
  info: CreatorInfoSchema.default({}),
  variables: z.record(z.string().startsWith('--'), z.string()).default({}),
  seelenweg: z.object({
    backgroundLayers: backgroundLayersSchema,
    items: z.object({
      backgroundLayers: backgroundLayersSchema,
    }).default({}),
    contextMenu: z.object({
      backgroundLayers: backgroundLayersSchema,
    }).default({}),
    preview: z.object({
      backgroundLayers: backgroundLayersSchema,
    }).default({}),
  }).default({}),
});

export type Theme = modify<inner, {
  info: modify<inner['info'], {
    filename: string;
    cssFileUrl: string | null;
  }>;
}>;