import { FallbackNode, HorizontalBranchNode, LeafNode, StackNode, VerticalBranchNode } from '../../../shared/schemas/Layout';

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

export type BranchNode = HorizontalBranchNode | VerticalBranchNode;
export type Node = LeafNode | FallbackNode | BranchNode | StackNode;

export const MAX_ALLOWED_ELEMENTS_PER_ROW = 10;
