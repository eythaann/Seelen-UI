import { cx } from '../../../../utils/styles';
import { FallbackContainer } from './containers/fallback';
import { LeafContainer } from './containers/leaf';
import { useSelector } from 'react-redux';

import { SelectCurrentWorkspace, Selectors } from '../../shared/store/app';
import { NodeImpl } from '../app';

import { Node } from '../domain';

import './index.css';

export function Container({ container }: { container: Node }) {
  const node = NodeImpl.from(container);

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
        className={cx('wm-container', `wm-${container.type.toLowerCase()}`)}
      >
        {node.inner.children.map((child) => (
          <Container key={child.priority} container={child} />
        ))}
      </div>
    );
  }

  return null;
}

export function Layout() {
  const workpsace = useSelector(SelectCurrentWorkspace);
  const version = useSelector(Selectors.version);

  if (!workpsace) {
    return null;
  }

  return <Container key={version} container={workpsace.layout.structure} />;
}
