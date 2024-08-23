import { AppBarHideMode } from './Seelenweg';
import z from 'zod';

export const FancyToolbarSchema = z.object({
  enabled: z.boolean().default(true),
  height: z.number().positive().default(30),
  placeholder: z.string().nullable().default(null),
  hideMode: z.nativeEnum(AppBarHideMode).default(AppBarHideMode.Never),
});

export type FancyToolbar = z.infer<typeof FancyToolbarSchema>;
