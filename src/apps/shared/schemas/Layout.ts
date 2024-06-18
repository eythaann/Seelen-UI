import { CreatorInfoSchema } from '.';
import { modify } from 'readable-types/dist';
import z from 'zod';

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

export const hwndSchema = z.number().nonnegative().describe('Window handle');

export type BaseNode = z.infer<typeof BaseNodeSchema>;
const BaseNodeSchema = z.object({
  type: z.nativeEnum(NodeType),
  subtype: z.nativeEnum(NodeSubtype).default(NodeSubtype.Permanent),
  priority: z
    .number()
    .positive()
    .describe('Order in how the tree will be traversed (1 = first, 2 = second, etc.)')
    .default(1),
  growFactor: z.number().describe('How much of the remaining space this node will take').default(1),
  condition: z.string().optional().nullable().describe('Math Condition for the node to be shown, e.g: n >= 3'),
});

export type StackNode = z.infer<typeof StackNodeSchema>;
const StackNodeSchema = BaseNodeSchema.extend({
  type: z.literal(NodeType.Stack),
  active: hwndSchema.nullable().default(null),
  handles: z.array(hwndSchema).default([]),
});

export type FallbackNode = z.infer<typeof FallbackNodeSchema>;
const FallbackNodeSchema = BaseNodeSchema.extend({
  type: z.literal(NodeType.Fallback),
  subtype: z.literal(NodeSubtype.Permanent).default(NodeSubtype.Permanent),
  active: hwndSchema.nullable().default(null),
  handles: z.array(hwndSchema).default([]),
});

export type LeafNode = z.infer<typeof LeafNodeSchema>;
const LeafNodeSchema = BaseNodeSchema.extend({
  type: z.literal(NodeType.Leaf),
  handle: hwndSchema.nullable().default(null),
});

export type HorizontalBranchNode = BaseNode & { type: NodeType.Horizontal; children: Node[] };
const HorizontalBranchNodeSchema = BaseNodeSchema.extend({
  type: z.literal(NodeType.Horizontal),
  children: z.array(z.lazy(() => NodeSchema)).min(1),
}) as z.ZodType<HorizontalBranchNode>;

export type VerticalBranchNode = BaseNode & { type: NodeType.Vertical; children: Node[] };
const VerticalBranchNodeSchema = BaseNodeSchema.extend({
  type: z.literal(NodeType.Vertical),
  children: z.array(z.lazy(() => NodeSchema)).min(1),
}) as z.ZodType<VerticalBranchNode>;

export type Node = z.infer<typeof NodeSchema>;
export const NodeSchema = z.union([
  StackNodeSchema,
  FallbackNodeSchema,
  LeafNodeSchema,
  HorizontalBranchNodeSchema,
  VerticalBranchNodeSchema,
]).describe('The layout tree');

type InnerLayout = z.infer<typeof LayoutSchema>;
export const LayoutSchema = z.object({
  info: CreatorInfoSchema.default({}),
  structure: NodeSchema.default({ type: NodeType.Fallback }),
  no_fallback_behavior: z.nativeEnum(NoFallbackBehavior).optional().nullable(),
});

export interface Layout {
  info: modify<InnerLayout['info'], {
    filename: string;
  }>;
  structure: InnerLayout['structure'];
  noFallbackBehavior: InnerLayout['no_fallback_behavior'];
}