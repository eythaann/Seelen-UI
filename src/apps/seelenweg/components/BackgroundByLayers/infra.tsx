import { HTMLAttributes, PropsWithChildren } from 'react';

import cs from './infra.module.css';

import { cx } from '../../../shared/styles';

interface PropsV2 extends PropsWithChildren, HTMLAttributes<HTMLDivElement> {
  className?: string;
  /** for backward compatibility */
  prefix?: string;
}

export function BackgroundByLayersV2({ children, className, prefix, ...divProps }: PropsV2) {
  let background = (
    <div className={cx(cs.background, 'bg-layers')}>
      {Array.from({ length: 10 }, (_, index) => (
        <div
          key={index}
          className={
            prefix
              ? cx(cs.layer, `bg-layer-${index + 1}`, `${prefix}-bg-layer-${index + 1}`)
              : cx(cs.layer, `bg-layer-${index + 1}`)
          }
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
