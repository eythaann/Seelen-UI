import { FancyToolbar, FancyToolbarSchema } from './FancyToolbar';
import { Monitor, MonitorSchema } from './Monitors';
import { Seelenweg, SeelenWegSchema } from './Seelenweg';
import { WindowManager, WindowManagerSchema } from './WindowManager';
import z from 'zod';

export const SettingsSchema = z.object({
  fancy_toolbar: FancyToolbarSchema.default({}),
  seelenweg: SeelenWegSchema.default({}),
  window_manager: WindowManagerSchema.default({}),
  monitors: z.array(MonitorSchema).min(1).default([MonitorSchema.parse({})]),
  ahk_enabled: z.boolean().default(true),
  selected_theme: z.string().nullable().default(null),
});

type inner = z.infer<typeof SettingsSchema> & {};
export interface ISettings {
  fancyToolbar: FancyToolbar;
  seelenweg: Seelenweg;
  windowManager: WindowManager;
  monitors: Monitor[];
  ahkEnabled: inner['ahk_enabled'];
  selectedTheme: inner['selected_theme'];
}