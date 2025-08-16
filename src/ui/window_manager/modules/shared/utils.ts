import { WmNodeKind } from '@seelen-ui/lib';
import { WmNode } from '@seelen-ui/lib/types';

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
}
