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

export interface BranchNode extends INode {
  type: NodeType.Horizontal | NodeType.Vertical;
  children: Node[];
}

export type Node = LeafNode | FallbackNode | BranchNode | StackNode;

export type Layout = {
  floating: HWND[];
  /** Layout can be monocontainer: FallbackNode or a Tree: BranchNode */
  structure: FallbackNode | BranchNode;
};

export const MAX_ALLOWED_ELEMENTS_PER_ROW = 10;
