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

export enum NoFallbackBehavior {
  Float = 'Float',
  Unmanaged = 'Unmanaged',
}

export interface WManagerLayoutInfo {
  displayName: string;
  author: string;
  description: string;
  filename: string;
}

export interface WmNodeBase {
  subtype: NodeSubtype;
  priority: number;
  growFactor: number;
  condition: string | null;
}

export interface WmVerticalNode extends WmNodeBase {
  type: NodeType.Vertical;
  children: WmNode[];
}

export interface WmHorizontalNode extends WmNodeBase {
  type: NodeType.Horizontal;
  children: WmNode[];
}

export interface WmLeafNode extends WmNodeBase {
  type: NodeType.Leaf;
  handle: number | null;
}

export interface WmStackNode extends WmNodeBase {
  type: NodeType.Stack;
  active: number | null;
  handles: number[];
}

export interface WmFallbackNode extends WmNodeBase {
  type: NodeType.Fallback;
  active: number | null;
  handles: number[];
}

export type WmNode = WmVerticalNode | WmHorizontalNode | WmLeafNode | WmStackNode | WmFallbackNode;

export interface WindowManagerLayout {
  info: WManagerLayoutInfo;
  structure: WmNode;
  noFallbackBehavior: NoFallbackBehavior;
}
