import { Popover, PopoverProps } from 'antd';
import { useState } from 'react';

import { CustomAnimationProps } from '../domain';

import { useTimeout } from '../../../hooks';
import { cx } from '../../../styles';

export interface AnimatedPopoverProps extends PopoverProps {
  animationDescription: CustomAnimationProps;
}

export function AnimatedPopover({ children, open, content, animationDescription, ...popoverProps }: AnimatedPopoverProps) {
  const [delayedOpenPopover, setDelayedOpenPopover] = useState(false);

  useTimeout(() => {
    setDelayedOpenPopover(!!open);
  }, animationDescription.maxAnimationTimeMs, [open]);

  const animationObject = {};
  if (animationDescription.openAnimationName) {
    animationObject[animationDescription.openAnimationName] = open && !delayedOpenPopover;
  }
  if (animationDescription.closeAnimationName) {
    animationObject[animationDescription.closeAnimationName] = delayedOpenPopover && !open;
  }

  return (
    <Popover
      open={open || delayedOpenPopover}
      {...popoverProps}
      content={content &&
        <div className={cx(animationObject)}>
          <>{content}</>
        </div>
      }
    >
      {children}
    </Popover>
  );
}