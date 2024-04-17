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

interface INode {
  type: NodeType;
  subtype: NodeSubtype;
  priority: number;
}

export interface StackNode extends INode {
  type: NodeType.Stack;
  active: number | null;
  handles: number[];
}

export interface FallbackNode extends INode {
  type: NodeType.Fallback;
  subtype: NodeSubtype.Permanent;
  active: number | null;
  handles: number[];
}

export interface LeafNode extends INode {
  type: NodeType.Leaf;
  handle: number | null;
}

export interface BranchNode extends INode {
  type: NodeType.Horizontal | NodeType.Vertical;
  children: Node[];
}

export type Node = LeafNode | FallbackNode | BranchNode | StackNode;

export type Layout = {
  floating: number[];
  /** Layout can be monocontainer: FallbackNode or a Tree: BranchNode */
  structure: FallbackNode | BranchNode;
};