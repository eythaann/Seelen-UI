import { Popover, PopoverProps } from 'antd';
import { useEffect, useState } from 'react';

import { useTimeout } from '../../hooks';
import { cx } from '../../styles';

export interface PopoverAnimationProps {
  maxAnimationTimeMs: number;
  openAnimationName: String;
  closeAnimationName: String;
}

export interface AnimatedPopoverProps extends PopoverProps {
  animationDescription: PopoverAnimationProps;
}

export default function AnimatedPopover({ children, open, onOpenChange, content, animationDescription, ...popoverProps }: AnimatedPopoverProps) {
  const [openPopover, setOpenPopover] = useState(!!open);
  const [delayedOpenPopover, setDelayedOpenPopover] = useState(false);

  useEffect(() => {
    setOpenPopover(!!open);
  }, [open]);

  useTimeout(() => {
    setDelayedOpenPopover(openPopover);
  }, animationDescription.maxAnimationTimeMs, [openPopover]);

  useEffect(() => {
    if (onOpenChange) {
      onOpenChange(delayedOpenPopover);
    }
  }, [delayedOpenPopover]);

  const animationObject = {};
  if (animationDescription.openAnimationName) {
    animationObject[animationDescription.openAnimationName] = openPopover && !delayedOpenPopover;
  }
  if (animationDescription.closeAnimationName) {
    animationObject[animationDescription.closeAnimationName] = delayedOpenPopover && !openPopover;
  }

  return (
    <Popover
      open={openPopover || delayedOpenPopover}
      onOpenChange={setOpenPopover}
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