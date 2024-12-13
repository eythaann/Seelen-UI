import { Dropdown, DropdownProps } from 'antd';
import { useState } from 'react';

import { CustomAnimationProps } from '../domain';

import { useTimeout } from '../../../hooks';
import { cx } from '../../../styles';

export interface AnimatedDropwonProps extends DropdownProps {
  animationDescription: CustomAnimationProps;
}

export function AnimatedDropdown({ children, open, onOpenChange, dropdownRender, animationDescription, ...dropdownProps }: AnimatedDropwonProps) {
  const [delayedOpenPopover, setDelayedOpenPopover] = useState(false);
  const [openReplacement, setOpenReplacement] = useState(false);

  useTimeout(() => {
    setDelayedOpenPopover((open || openReplacement));
  }, animationDescription.maxAnimationTimeMs, [open || openReplacement]);

  const animationObject = {};
  if (animationDescription.openAnimationName) {
    animationObject[animationDescription.openAnimationName] = (open || openReplacement) && !delayedOpenPopover;
  }
  if (animationDescription.closeAnimationName) {
    animationObject[animationDescription.closeAnimationName] = delayedOpenPopover && !(open || openReplacement);
  }

  return (
    <Dropdown
      open={open || openReplacement || delayedOpenPopover}
      onOpenChange={(open, info) => {
        if (onOpenChange) {
          onOpenChange(open, info);
        } else {
          setOpenReplacement(open);
        }
      }}
      {...dropdownProps}
      dropdownRender={(origin) => dropdownRender &&
        <div className={cx(animationObject)}>
          {dropdownRender(origin)}
        </div>
      }
    >
      {children}
    </Dropdown>
  );
}