import { CSSProperties, memo } from 'react';

import cs from './infra.module.css';

interface SeelenWegBackgroundProps {
  styles: CSSProperties[];
}
export const BackgroundByLayers = memo(({ styles }: SeelenWegBackgroundProps) => {
  return <div className={cs.backgroundLayers}>
    {styles.map((layer, index) => (
      <div key={index} className={cs.layer} style={layer} />
    ))}
  </div>;
});