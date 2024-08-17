import { cx } from '../../../shared/styles';
import { HTMLAttributes, PropsWithChildren } from 'react';

import cs from './infra.module.css';

interface PropsV2 extends PropsWithChildren, HTMLAttributes<HTMLDivElement> {
  className?: string;
  /** for backward compatibility */
  prefix?: string;
}

export function BackgroundByLayersV2({
  children,
  className,
  prefix: _prefix,
  ...divProps
}: PropsV2) {
  const prefix = _prefix ? _prefix + '-' : '';

  let background = (
    <div className={cs.background}>
      {Array.from({ length: 10 }, (_, index) => (
        <div
          key={index}
          className={cx(cs.layer, `${prefix}bg-layer-${index + 1}`, `bg-layer-${index + 1}`)}
        />
      ))}
    </div>
  );

  if (!children) {
    /** for backward compatibility with V1 */
    return background;
  }

  return (
    <div className={cx(cs.container, className)} {...divProps}>
      {background}
      {children}
    </div>
  );
}
