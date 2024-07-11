import { cx } from '../../../shared/styles';
import { CSSProperties, memo, PropsWithChildren } from 'react';

import cs from './infra.module.css';

interface SeelenWegBackgroundProps {
  layers: CSSProperties[] | number;
  prefix: string;
}
export const BackgroundByLayers = memo(({ prefix, layers: styles }: SeelenWegBackgroundProps) => {
  const layerStyles = typeof styles === 'object' ? styles : new Array(styles).fill({});

  return (
    <div className={cx(cs.background)}>
      {layerStyles.map((layer, index) => (
        <div
          key={index}
          className={cx(cs.layer, `${prefix}-bg-layer-${index + 1}`)}
          style={layer}
        />
      ))}
    </div>
  );
});

interface PropsV2 extends PropsWithChildren {
  className?: string;
  bgPrefix?: string;
  amount: number;
}

export function BackgroundByLayersV2({ amount, children, className, bgPrefix: _prefix }: PropsV2) {
  const prefix = _prefix ? _prefix + '-' : '';
  return (
    <div className={cx(cs.container, className)}>
      <div className={cs.background}>
        {Array.from({ length: amount }, (_, index) => (
          <div key={index} className={cx(cs.layer, `${prefix}bg-layer-${index + 1}`)} />
        ))}
      </div>
      {children}
    </div>
  );
}
