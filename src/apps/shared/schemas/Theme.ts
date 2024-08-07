import { CreatorInfoSchema } from '.';
import { modify } from 'readable-types';
import { z } from 'zod';

type inner = z.infer<typeof ThemeSchema>;
export const ThemeSchema = z.object({
  info: CreatorInfoSchema.extend({
    tags: z.array(z.string()).default([]),
  }).default({}),
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
