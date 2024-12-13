import { Dropdown, DropdownProps } from 'antd';
import { useState } from 'react';

import { CustomAnimationProps } from '../domain';

import { useTimeout } from '../../../hooks';
import { cx } from '../../../styles';

export interface AnimatedDropwonProps extends DropdownProps {
  animationDescription: CustomAnimationProps;
}

export function AnimatedDropdown({ children, open, dropdownRender, animationDescription, ...dropdownProps }: AnimatedDropwonProps) {
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
    <Dropdown
      open={open || delayedOpenPopover}
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