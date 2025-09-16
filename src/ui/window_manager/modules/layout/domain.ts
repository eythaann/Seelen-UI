import { WmNode } from "@seelen-ui/lib/types";

export enum Reservation {
  Left = "Left",
  Right = "Right",
  Top = "Top",
  Bottom = "Bottom",
  Stack = "Stack",
  Float = "Float",
}

export enum Sizing {
  Increase = "Increase",
  Decrease = "Decrease",
}

export type WmFallbackNode = Extract<WmNode, { type: "Fallback" }>;
export type WmHorizontalNode = Extract<WmNode, { type: "Horizontal" }>;
export type WmVerticalNode = Extract<WmNode, { type: "Vertical" }>;
export type WmLeafNode = Extract<WmNode, { type: "Leaf" }>;
export type WmStackNode = Extract<WmNode, { type: "Stack" }>;

export type BranchNode = WmVerticalNode | WmHorizontalNode;
export type Node = WmNode;

export const MAX_ALLOWED_ELEMENTS_PER_ROW = 10;
