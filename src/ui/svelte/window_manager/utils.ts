import { type WmNode, WmNodeKind } from "@seelen-ui/lib/types";

export class NodeUtils {
  static isEmpty(node: WmNode): boolean {
    switch (node.type) {
      case WmNodeKind.Leaf:
        return !node.active;
      case WmNodeKind.Stack:
        return node.windows.length === 0;
      case WmNodeKind.Horizontal:
      case WmNodeKind.Vertical:
        return node.children.every(NodeUtils.isEmpty);
    }
  }

  static contains(node: WmNode, searchingWindow: number): boolean {
    switch (node.type) {
      case WmNodeKind.Leaf:
        return node.active === searchingWindow;
      case WmNodeKind.Stack:
        return node.windows.includes(searchingWindow);
      case WmNodeKind.Horizontal:
      case WmNodeKind.Vertical:
        return node.children.some((child) => NodeUtils.contains(child, searchingWindow));
    }
  }

  static some(node: WmNode, predicate: (window: number) => boolean): boolean {
    switch (node.type) {
      case WmNodeKind.Leaf:
        return predicate(node.active!);
      case WmNodeKind.Stack:
        if (node.active) {
          return predicate(node.active);
        }
        return false;
      case WmNodeKind.Horizontal:
      case WmNodeKind.Vertical:
        return node.children.some((child) => NodeUtils.some(child, predicate));
    }
  }
}
