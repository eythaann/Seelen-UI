import { Obtainable, SeelenCommand, SeelenEvent } from '../handlers';

export * from './theme';
export * from './settings';
export * from './weg_items';
export * from './wm_layout';
export * from './placeholder';
export * from './settings_by_app';
export * from './settings_by_monitor';
export * from './icon_pack';
export * from './plugin';

export interface LauncherHistory {
  [x: string]: string[];
}
export const LauncherHistory = Obtainable<LauncherHistory>(
  SeelenCommand.StateGetHistory,
  SeelenEvent.StateHistoryChanged,
);

export class ResourceMetadata {
  displayName: string = 'Unknown';
  author: string = 'Unknown';
  description: string = '';
  filename: string = '';
  tags: string[] = [];
}