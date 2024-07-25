import { FancyToolbar, FancyToolbarSchema } from './FancyToolbar';
import { Monitor, MonitorSchema } from './Monitors';
import { Seelenweg, SeelenWegSchema } from './Seelenweg';
import { WindowManager, WindowManagerSchema } from './WindowManager';
import z from 'zod';

export type AhkVariables = Record<string, AhkVar>;
export type AhkVar = z.infer<typeof AhkVarSchema>;
export const AhkVarSchema = z.object({
  fancy: z.string(),
  ahk: z.string(),
});

export const AhkVariablesSchema = z.object({
  // open_settings: AhkVarSchema.default({ fancy: 'Win + K', ahk: '#k' }),
  // pause_wm: AhkVarSchema.default({ fancy: 'Win + Control + Alt + P', ahk: '^#!p' }),
  // reservations
  reserve_top: AhkVarSchema.default({ fancy: 'Win + Shift + I', ahk: '#+i' }),
  reserve_bottom: AhkVarSchema.default({ fancy: 'Win + Shift + K', ahk: '#+k' }),
  reserve_left: AhkVarSchema.default({ fancy: 'Win + Shift + J', ahk: '#+j' }),
  reserve_right: AhkVarSchema.default({ fancy: 'Win + Shift + L', ahk: '#+l' }),
  reserve_float: AhkVarSchema.default({ fancy: 'Win + Shift + U', ahk: '#+u' }),
  reserve_stack: AhkVarSchema.default({ fancy: 'Win + Shift + O', ahk: '#+o' }),
  // focus
  focus_top: AhkVarSchema.default({ fancy: 'Win + Shift + W', ahk: '#+w' }),
  focus_bottom: AhkVarSchema.default({ fancy: 'Win + Shift + S', ahk: '#+s' }),
  focus_left: AhkVarSchema.default({ fancy: 'Win + Shift + A', ahk: '#+a' }),
  focus_right: AhkVarSchema.default({ fancy: 'Win + Shift + D', ahk: '#+d' }),
  focus_latest: AhkVarSchema.default({ fancy: 'Win + Shift + E', ahk: '#+e' }),
  // window size
  increase_width: AhkVarSchema.default({ fancy: 'Win + Alt + =', ahk: '#!=' }),
  decrease_width: AhkVarSchema.default({ fancy: 'Win + Alt + -', ahk: '#!-' }),
  increase_height: AhkVarSchema.default({ fancy: 'Win + Shift + =', ahk: '#+=' }),
  decrease_height: AhkVarSchema.default({ fancy: 'Win + Shift + -', ahk: '#+-' }),
  restore_sizes: AhkVarSchema.default({ fancy: 'Win + Alt + 0', ahk: '#!0' }),
  // switch
  switch_workspace_0: AhkVarSchema.default({ fancy: 'Alt + 1', ahk: '!1' }),
  switch_workspace_1: AhkVarSchema.default({ fancy: 'Alt + 2', ahk: '!2' }),
  switch_workspace_2: AhkVarSchema.default({ fancy: 'Alt + 3', ahk: '!3' }),
  switch_workspace_3: AhkVarSchema.default({ fancy: 'Alt + 4', ahk: '!4' }),
  switch_workspace_4: AhkVarSchema.default({ fancy: 'Alt + 5', ahk: '!5' }),
  switch_workspace_5: AhkVarSchema.default({ fancy: 'Alt + 6', ahk: '!6' }),
  switch_workspace_6: AhkVarSchema.default({ fancy: 'Alt + 7', ahk: '!7' }),
  switch_workspace_7: AhkVarSchema.default({ fancy: 'Alt + 8', ahk: '!8' }),
  switch_workspace_8: AhkVarSchema.default({ fancy: 'Alt + 9', ahk: '!9' }),
  switch_workspace_9: AhkVarSchema.default({ fancy: 'Alt + 0', ahk: '!0' }),
  // move
  move_to_workspace_0: AhkVarSchema.default({ fancy: 'Alt + Shift + 1', ahk: '!+1' }),
  move_to_workspace_1: AhkVarSchema.default({ fancy: 'Alt + Shift + 2', ahk: '!+2' }),
  move_to_workspace_2: AhkVarSchema.default({ fancy: 'Alt + Shift + 3', ahk: '!+3' }),
  move_to_workspace_3: AhkVarSchema.default({ fancy: 'Alt + Shift + 4', ahk: '!+4' }),
  move_to_workspace_4: AhkVarSchema.default({ fancy: 'Alt + Shift + 5', ahk: '!+5' }),
  move_to_workspace_5: AhkVarSchema.default({ fancy: 'Alt + Shift + 6', ahk: '!+6' }),
  move_to_workspace_6: AhkVarSchema.default({ fancy: 'Alt + Shift + 7', ahk: '!+7' }),
  move_to_workspace_7: AhkVarSchema.default({ fancy: 'Alt + Shift + 8', ahk: '!+8' }),
  move_to_workspace_8: AhkVarSchema.default({ fancy: 'Alt + Shift + 9', ahk: '!+9' }),
  move_to_workspace_9: AhkVarSchema.default({ fancy: 'Alt + Shift + 0', ahk: '!+0' }),
  // send
  send_to_workspace_0: AhkVarSchema.default({ fancy: 'Win + Shift + 1', ahk: '#+1' }),
  send_to_workspace_1: AhkVarSchema.default({ fancy: 'Win + Shift + 2', ahk: '#+2' }),
  send_to_workspace_2: AhkVarSchema.default({ fancy: 'Win + Shift + 3', ahk: '#+3' }),
  send_to_workspace_3: AhkVarSchema.default({ fancy: 'Win + Shift + 4', ahk: '#+4' }),
  send_to_workspace_4: AhkVarSchema.default({ fancy: 'Win + Shift + 5', ahk: '#+5' }),
  send_to_workspace_5: AhkVarSchema.default({ fancy: 'Win + Shift + 6', ahk: '#+6' }),
  send_to_workspace_6: AhkVarSchema.default({ fancy: 'Win + Shift + 7', ahk: '#+7' }),
  send_to_workspace_7: AhkVarSchema.default({ fancy: 'Win + Shift + 8', ahk: '#+8' }),
  send_to_workspace_8: AhkVarSchema.default({ fancy: 'Win + Shift + 9', ahk: '#+9' }),
  send_to_workspace_9: AhkVarSchema.default({ fancy: 'Win + Shift + 0', ahk: '#+0' }),
});

export const SettingsSchema = z.object({
  fancy_toolbar: FancyToolbarSchema.default({}),
  seelenweg: SeelenWegSchema.default({}),
  window_manager: WindowManagerSchema.default({}),
  monitors: z
    .array(MonitorSchema)
    .min(1)
    .default([MonitorSchema.parse({})]),
  ahk_enabled: z.boolean().default(true),
  ahk_variables: AhkVariablesSchema.default({}),
  selected_theme: z
    .union([z.string(), z.array(z.string())])
    .transform((arg) => {
      // backward compatibility with versions before 1.4.0
      if (arg === 'default.json') {
        return ['default'];
      }
      return Array.isArray(arg) ? arg : [arg];
    })
    .default(['default']),
  dev_tools: z.boolean().default(false),
  language: z.string().default(() => {
    if (globalThis.navigator) {
      return globalThis.navigator.language.split('-')[0] || 'en';
    }
    return 'en';
  }),
});

export interface ISettings {
  fancyToolbar: FancyToolbar;
  seelenweg: Seelenweg;
  windowManager: WindowManager;
  monitors: Monitor[];
  ahkEnabled: boolean;
  ahkVariables: AhkVariables;
  selectedTheme: string[];
  devTools: boolean;
  language: string;
}
