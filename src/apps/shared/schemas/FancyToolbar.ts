import z from 'zod';

export const FancyToolbarSchema = z.object({
  enabled: z.boolean().default(true),
  height: z.number().positive().default(30),
  placeholder: z.string().nullable().default(null),
});

export type FancyToolbar = z.infer<typeof FancyToolbarSchema>;
