import { HWND } from '../shared/utils/domain';

export enum NodeType {
  Vertical = 'Vertical',
  Horizontal = 'Horizontal',
  Leaf = 'Leaf',
  Stack = 'Stack',
  Fallback = 'Fallback',
}

export enum NodeSubtype {
  Temporal = 'Temporal',
  Permanent = 'Permanent',
}

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

interface INode {
  type: NodeType;
  subtype: NodeSubtype;
  priority: number;
  growFactor: number;
}

export interface StackNode extends INode {
  type: NodeType.Stack;
  active: HWND | null;
  handles: HWND[];
}

export interface FallbackNode extends INode {
  type: NodeType.Fallback;
  subtype: NodeSubtype.Permanent;
  active: HWND | null;
  handles: HWND[];
}

export interface LeafNode extends INode {
  type: NodeType.Leaf;
  handle: HWND | null;
}

export interface HorizontalBranchNode extends INode {
  type: NodeType.Horizontal;
  children: Node[];
}

export interface VerticalBranchNode extends INode {
  type: NodeType.Vertical;
  children: Node[];
}

export type BranchNode = HorizontalBranchNode | VerticalBranchNode;
export type Node = LeafNode | FallbackNode | BranchNode | StackNode;

export type Layout = {
  /**
   * Tree: BranchNode
   * Monocontainer: FallbackNode
   * FirstManaged: LeafNode
   * FirstManaged + Stack: StackNode
  */
  structure: Node;
  /** What to do if a FallbackNode is not present */
  noFallbackBehavior?: 'Float' | 'Unmanaged';
};

export const MAX_ALLOWED_ELEMENTS_PER_ROW = 10;
