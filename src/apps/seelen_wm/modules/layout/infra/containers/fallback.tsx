import { useSelector } from 'react-redux';
import { WmFallbackNode } from 'seelen-core';

import { Selectors } from '../../../shared/store/app';

import { cx } from '../../../../../shared/styles';
import { LeafContainer } from './leaf';

interface Props {
  node: WmFallbackNode;
}

export function FallbackContainer({ node }: Props) {
  const { border } = useSelector(Selectors.settings);

  return (
    <div
      style={{
        flexGrow: node.growFactor,
      }}
      className={cx('wm-container', 'wm-stack')}
    >
      {node.handles.length > 1 && (
        <div
          className={cx('wm-stack-bar', {
            'wm-stack-bar-with-borders': border.enabled,
          })}
        >
          {node.handles.map((handle) => (
            <div key={handle} className="wm-stack-bar-item">
              {handle}
            </div>
          ))}
        </div>
      )}
      {node.active && <LeafContainer hwnd={node.active} />}
    </div>
  );
}
