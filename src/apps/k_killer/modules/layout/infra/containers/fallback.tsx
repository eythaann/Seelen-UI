import { toPhysicalPixels } from '../../../../../utils';
import { cx } from '../../../../../utils/styles';
import { LeafContainer } from './leaf';
import { ReservedContainer } from './reserved';
import { invoke } from '@tauri-apps/api/core';
import { useCallback, useEffect, useRef } from 'react';
import { useSelector } from 'react-redux';

import { Selectors } from '../../../shared/store/app';

import { FallbackNode } from '../../domain';

import cs from '../index.module.css';

interface Props {
  node: FallbackNode;
}

export function FallbackContainer({ node }: Props) {
  return (
    <div
      style={{
        flexGrow: node.growFactor,
      }}
      className={cx(cs.container, cs.stack)}
    >
      <div className={cs.stackedBar}>
        {node.handles.map((handle) => (
          <div className={cs.stackedItem}>{handle}</div>
        ))}
      </div>
      {node.active && <LeafContainer hwnd={node.active} />}
    </div>
  );
}
