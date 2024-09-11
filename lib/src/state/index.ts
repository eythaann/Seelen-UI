import { Obtainable, SeelenCommand, SeelenEvent } from '../handlers';

export * from './theme';
export * from './settings';
export * from './weg_items';
export * from './wm_layout';
export * from './placeholder';
export * from './settings_by_app';

export interface LauncherHistory {
  [x: string]: string[];
}
export const LauncherHistory = Obtainable<LauncherHistory>(
  SeelenCommand.StateGetHistory,
  SeelenEvent.StateHistoryChanged,
);
