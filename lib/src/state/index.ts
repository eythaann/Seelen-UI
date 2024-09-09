import { EventHandler, InvokeHandler, Obtainable } from '../handlers';

export * from './theme';
export * from './settings';
export * from './weg_items';

export interface LauncherHistory {
  [x: string]: string[];
}
export const LauncherHistory = Obtainable<LauncherHistory>(
  InvokeHandler.GetHistory,
  EventHandler.History,
);
