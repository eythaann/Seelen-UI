import { cx } from '@shared/styles';
import { useSelector } from 'react-redux';

import { Selectors } from '../../shared/store/app';
import { NodeImpl } from '../app';

import { Node } from '../domain';

import { FallbackContainer } from './containers/fallback';
import { LeafContainer } from './containers/leaf';

import './index.css';

export function Container({ node: _node }: { node: Node }) {
  const node = NodeImpl.from(_node);

  if (node.isEmpty()) {
    return null;
  }

  if (node.isFallback()) {
    return <FallbackContainer node={node.inner} />;
  }

  if (node.isLeaf() && node.inner.handle) {
    return <LeafContainer hwnd={node.inner.handle} growFactor={node.inner.growFactor} />;
  }

  if (node.isBranch()) {
    return (
      <div
        style={{
          flexGrow: node.inner.growFactor,
        }}
        className={cx('wm-container', `wm-${_node.type.toLowerCase()}`)}
      >
        {node.inner.children.map((child, idx) => (
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
