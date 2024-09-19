import { WmFallbackNode, WmHorizontalNode, WmLeafNode, WmStackNode, WmVerticalNode } from 'seelen-core';

export enum Reservation {
  Left = 'Left',
  Right = 'Right',
  Top = 'Top',
  Bottom = 'Bottom',
  Stack = 'Stack',
  Float = 'Float',
}

export enum Sizing {
  Increase = 'Increase',
  Decrease = 'Decrease',
}

export type BranchNode = WmHorizontalNode | WmVerticalNode;
export type Node = WmLeafNode | WmFallbackNode | BranchNode | WmStackNode;

export const MAX_ALLOWED_ELEMENTS_PER_ROW = 10;
