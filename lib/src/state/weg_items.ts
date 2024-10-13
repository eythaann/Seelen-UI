export enum SwItemType {
  Pinned = 'Pinned',
  TemporalApp = 'TemporalPin',
  Separator = 'Separator',
  Media = 'Media',
  Start = 'StartMenu',
}

export interface PinnedWegItem {
  type: SwItemType.Pinned;
  path: string;
  execution_command: string;
  execution_arguments: null | string;
  is_dir: boolean;
}

export interface TemporalPinnedWegItem {
  type: SwItemType.TemporalApp;
  path: string;
  execution_command: string;
  execution_arguments: null | string;
  is_dir: boolean;
}

export interface SeparatorWegItem {
  type: SwItemType.Separator;
  id: string;
}

export interface MediaWegItem {
  type: SwItemType.Media;
}

export interface StartWegItem {
  type: SwItemType.Start;
}

export type WegItem =
  | PinnedWegItem
  | TemporalPinnedWegItem
  | SeparatorWegItem
  | MediaWegItem
  | StartWegItem;

export interface WegItems {
  left: WegItem[];
  center: WegItem[];
  right: WegItem[];
}
