import { cx } from '../../../shared/styles';
import { motion, useAnimationControls } from 'framer-motion';
import { PropsWithChildren } from 'react';
import { useSelector } from 'react-redux';

import { newSelectors } from '../../modules/shared/store/app/reducer';

import cs from './index.module.css';

interface Props extends PropsWithChildren, React.HTMLAttributes<HTMLDivElement> {}

export function Monitor({ children, className, ...props }: Props) {
  const wallpaper = useSelector(newSelectors.wallpaper);
  const controls = useAnimationControls();

  return (
    <div className={cx(cs.monitor, className)} {...props}>
      <div className={cs.screen}>
        {wallpaper && (
          <motion.img
            className={cs.wallpaper}
            src={wallpaper}
            initial={{ opacity: 0 }}
            animate={controls}
            onLoad={() => {
              controls.start({ opacity: 1, transition: { duration: 0.3, ease: 'linear' } });
            }}
          />
        )}
        {children}
      </div>
    </div>
  );
}
