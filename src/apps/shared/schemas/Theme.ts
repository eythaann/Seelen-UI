import { CreatorInfoSchema } from '.';
import { modify } from 'readable-types/dist';
import { z } from 'zod';

const backgroundLayersSchema = z.number().min(1).default(1);

type inner = z.infer<typeof ThemeSchema>;
export const ThemeSchema = z.object({
  info: CreatorInfoSchema.extend({
    tags: z.array(z.string()).default([]),
  }).default({}),
  variables: z.record(z.string().startsWith('--'), z.string()).default({}),
  layers: z
    .object({
      weg: z
        .object({
          bg: backgroundLayersSchema,
          items: z
            .object({
              bg: backgroundLayersSchema,
            })
            .default({}),
          contextMenu: z
            .object({
              bg: backgroundLayersSchema,
            })
            .default({}),
          preview: z
            .object({
              bg: backgroundLayersSchema,
            })
            .default({}),
        })
        .default({}),
      toolbar: z
        .object({
          bg: backgroundLayersSchema,
          fastSettings: z
            .object({
              bg: backgroundLayersSchema,
            })
            .default({}),
          systemTray: z
            .object({
              bg: backgroundLayersSchema,
            })
            .default({}),
        })
        .default({}),
    })
    .default({}),
  styles: z
    .object({
      weg: z.string().default(''),
      toolbar: z.string().default(''),
      wm: z.string().default(''),
    })
    .default({}),
});

export type Theme = modify<
  inner,
  {
    info: modify<
      inner['info'],
      {
        filename: string;
      }
    >;
  }
>;
