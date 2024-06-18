import { cx } from '../../../shared/styles';
import { CSSProperties, memo } from 'react';

import cs from './infra.module.css';

interface SeelenWegBackgroundProps {
  layers: CSSProperties[] | number;
  prefix: string;
}
export const BackgroundByLayers = memo(({ prefix, layers: styles }: SeelenWegBackgroundProps) => {
  const layerStyles = typeof styles === 'object' ? styles : new Array(styles).fill({});

  return <div className={cx(cs.backgroundLayers)}>
    {layerStyles.map((layer, index) => (
      <div key={index} className={cx(cs.layer, `${prefix}-bg-layer-${index + 1}`)} style={layer} />
    ))}
  </div>;
});