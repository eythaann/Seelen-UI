import z from 'zod';

export const ModuleSchema = z.union([
  z.object({
    type: z.literal('text'),
    template: z.string(),
  }),
  z.object({
    type: z.literal('date'),
    each: z
      .union([z.literal('second'), z.literal('minute'), z.literal('hour'), z.literal('day')])
      .describe('Time unit to update showing the date')
      .default('second'),
    template: z
      .string()
      .describe('A format for the date used by MomentJS')
      .default('MMMM Do, h:mm:ss a'),
  }),
]);

export const FancyToolbarSchema = z.object({
  enabled: z.boolean().default(true),
  height: z.number().positive().default(30),
  modules: z
    .object({
      left: z.array(ModuleSchema).default([]),
      center: z.array(ModuleSchema).default([]),
      right: z.array(ModuleSchema).default([]),
    })
    .default({}),
});

export type FancyToolbar = z.infer<typeof FancyToolbarSchema>;
