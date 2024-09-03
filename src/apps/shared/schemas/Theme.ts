import { CreatorInfoSchema } from '.';
import { z } from 'zod';

export const ThemeSchema = z.object({
  info: CreatorInfoSchema.extend({
    tags: z.array(z.string()),
  }),
  styles: z.object({
    weg: z.string(),
    toolbar: z.string(),
    wm: z.string(),
    launcher: z.string(),
  }),
});

export interface Theme extends z.infer<typeof ThemeSchema> {}
