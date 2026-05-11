import { TwmNodeKind, type TwmRuntimeTree } from "@seelen-ui/lib/types";

export class NodeUtils {
  static isEmpty(tree: TwmRuntimeTree, nodeId: number): boolean {
    const node = tree.nodes[nodeId];
    if (!node) return true;
    switch (node.kind) {
      case TwmNodeKind.Leaf:
      case TwmNodeKind.Stack:
        return node.windows.length === 0;
      case TwmNodeKind.Horizontal:
      case TwmNodeKind.Vertical:
        return node.children.every((id) => NodeUtils.isEmpty(tree, id));
    }
  }

  static contains(tree: TwmRuntimeTree, nodeId: number, searchingWindow: number): boolean {
    const node = tree.nodes[nodeId];
    if (!node) return false;
    switch (node.kind) {
      case TwmNodeKind.Leaf:
      case TwmNodeKind.Stack:
        return node.windows.includes(searchingWindow);
      case TwmNodeKind.Horizontal:
      case TwmNodeKind.Vertical:
        return node.children.some((id) => NodeUtils.contains(tree, id, searchingWindow));
    }
  }

  static some(
    tree: TwmRuntimeTree,
    nodeId: number,
    predicate: (window: number) => boolean,
  ): boolean {
    const node = tree.nodes[nodeId];
    if (!node) return false;
    switch (node.kind) {
      case TwmNodeKind.Leaf:
      case TwmNodeKind.Stack: {
        const active = node.activeWindow;
        return active !== null && predicate(active);
      }
      case TwmNodeKind.Horizontal:
      case TwmNodeKind.Vertical:
        return node.children.some((id) => NodeUtils.some(tree, id, predicate));
    }
  }
}
