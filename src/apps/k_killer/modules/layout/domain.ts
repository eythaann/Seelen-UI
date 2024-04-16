export enum BoxType {
  Vertical = 'Vertical',
  Horizontal = 'Horizontal',
  Stack = 'Stack',
  Reserved = 'Reserved',
}

export enum BoxSubType {
  Temporal = 'Temporal',
  Permanent = 'Permanent',
}

interface Box {
  type: BoxType;
  subtype: BoxSubType;
  priority: number;
}

export interface StackedBox extends Box {
  type: BoxType.Stack;
  active: number | null;
  handles: number[];
}

export interface ReservedBox extends Box {
  type: BoxType.Reserved;
  handle?: number;
}

export interface SplittedBox extends Box {
  type: BoxType.Horizontal | BoxType.Vertical;
  children: Container[];
}

export type Container = ReservedBox | StackedBox | SplittedBox;

export type Layout = {
  floating: number[];
  structure: StackedBox | SplittedBox;
};