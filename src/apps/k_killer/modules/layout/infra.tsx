import { toPhysicalPixels } from '../../../utils';
import { cx } from '../../../utils/styles';
import { invoke } from '@tauri-apps/api/core';
import { isEqual } from 'lodash';
import { useCallback, useEffect, useRef } from 'react';
import { useSelector } from 'react-redux';

import { SelectCurrentWorkspace, Selectors } from '../shared/store/app';
import { isEmptyContainer } from './app';

import { BoxType, Container, Layout, ReservedBox } from './domain';

import cs from './index.module.css';

function ReservedContainer({ container }: { container: ReservedBox }) {
  const ref = useRef<HTMLDivElement>(null);

  const updateSize = useCallback(() => {
    if (!ref.current) {
      return;
    }
    const domRect = ref.current.getBoundingClientRect();
    invoke('set_window_position', {
      hwnd: container.handle,
      rect: {
        top: toPhysicalPixels(domRect.top),
        left: toPhysicalPixels(domRect.left),
        right: toPhysicalPixels(domRect.width),
        bottom: toPhysicalPixels(domRect.height),
      },
    });
  }, [container]);

  useEffect(updateSize);

  return <div ref={ref} className={cx(cs.container, cs.reserved)} />;
}

export function Container({ container }: { container: Container }) {
  if (container.type === BoxType.Stack && container.handles.length > 0) {
    return <div className={cx(cs.container, cs.stack)} />;
  }

  if (container.type === BoxType.Reserved && container.handle) {
    return <ReservedContainer container={container} />;
  }

  if (
    (container.type === BoxType.Horizontal || container.type === BoxType.Vertical) &&
    !isEmptyContainer(container)
  ) {
    return (
      <div className={cx(cs.container, cs[container.type])}>
        {container.children.map((child) => (
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
