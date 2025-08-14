import { WmNodeKind } from '@seelen-ui/lib';
import { cx } from '@shared/styles';
import { useSelector } from 'react-redux';

import { Selectors } from '../../shared/store/app';

import { Node } from '../domain';

import { Leaf } from './containers/leaf';
import { Stack } from './containers/stack';

import './index.css';

function isNodeEmpty(node: Node): boolean {
  switch (node.type) {
    case WmNodeKind.Leaf:
      return !node.active;
    case WmNodeKind.Stack:
      return node.windows.length === 0;
    case WmNodeKind.Horizontal:
    case WmNodeKind.Vertical:
      return node.children.every(isNodeEmpty);
  }
}

export function Container({ node }: { node: Node }) {
  if (isNodeEmpty(node)) {
    return null;
  }

  if (node.type === WmNodeKind.Stack) {
    return <Stack node={node} />;
  }

  if (node.type === WmNodeKind.Leaf && node.active) {
    return <Leaf hwnd={node.active} growFactor={node.growFactor} />;
  }

  if (node.type === WmNodeKind.Horizontal || node.type === WmNodeKind.Vertical) {
    return (
      <div
        style={{
          flexGrow: node.growFactor,
        }}
        className={cx('wm-container', `wm-${node.type.toLowerCase()}`)}
      >
        {node.children.map((child, idx) => (
          <Container key={idx} node={child} />
        ))}
      </div>
    );
  }

  return null;
}

export function Layout() {
  const layout = useSelector(Selectors.layout);

  if (!layout) {
    return null;
  }

  return <Container node={layout} />;
}
