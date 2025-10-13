import { Dropdown, type DropdownProps } from "antd";
import type { ComponentChild } from "preact";
import { useEffect, useState } from "react";

import type { LegacyCustomAnimationProps } from "../domain.ts";

import { useDebounce } from "../../../hooks.ts";
import { cx } from "../../../styles.ts";

export interface AnimatedDropwonProps extends DropdownProps {
  animationDescription: LegacyCustomAnimationProps;
  children: ComponentChild;
}

export function AnimatedDropdown({
  children,
  open: openProp,
  onOpenChange,
  dropdownRender,
  animationDescription,
  ...dropdownProps
}: AnimatedDropwonProps) {
  const [innerOpen, setInnerOpen] = useState(false);
  const [shouldRender, setShouldRender] = useState(false);

  const open = openProp ?? innerOpen;

  const { maxAnimationTimeMs = 500, openAnimationName, closeAnimationName } = animationDescription;

  const unrenderPopup = useDebounce(() => {
    setShouldRender(false);
  }, maxAnimationTimeMs);

  useEffect(() => {
    if (open) {
      setShouldRender(true);
      unrenderPopup.cancel();
    } else {
      unrenderPopup();
    }
  }, [open]);

  const classnames: Record<string, boolean> = {
    "sl-popup-open": open,
    "sl-popup-close": !open,
  };

  if (openAnimationName) {
    classnames[openAnimationName] = open;
  }

  if (closeAnimationName) {
    classnames[closeAnimationName] = !open;
  }

  return (
    <Dropdown
      open={open || shouldRender}
      onOpenChange={(open, event) => {
        if (open) {
          setShouldRender(open);
        }
        setInnerOpen(open);
        onOpenChange?.(open, event);
      }}
      {...dropdownProps}
      dropdownRender={(origin) => dropdownRender && <div className={cx(classnames)}>{dropdownRender(origin)}</div>}
    >
      {children}
    </Dropdown>
  );
}
