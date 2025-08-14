import { WmNode } from '@seelen-ui/lib/types';
import { cx } from '@shared/styles';
import { useSelector } from 'react-redux';

import { Selectors } from '../../../shared/store/app';

import { Leaf } from './leaf';

interface Props {
  node: WmNode;
}

export function Stack({ node }: Props) {
  const { border } = useSelector(Selectors.settings);

  return (
    <div
      style={{
        flexGrow: node.growFactor,
      }}
      className={cx('wm-container', 'wm-stack')}
    >
      {node.windows.length > 1 && (
        <div
          className={cx('wm-stack-bar', {
            'wm-stack-bar-with-borders': border.enabled,
          })}
        >
          {node.windows.map((handle) => (
            <div key={handle} className="wm-stack-bar-item">
              {handle}
            </div>
          ))}
        </div>
      )}
      {node.active && <Leaf hwnd={node.active} />}
    </div>
  );
}
