import type { IconName } from "libs/ui/react/components/Icon/icons.ts";
import type { HTMLAttributes } from "preact/compat";

import { cx } from "../../utils/styling.ts";
import InlineSVG from "../InlineSvg/index.tsx";
import cs from "./index.module.css";

interface ReactIconProps extends HTMLAttributes<HTMLElement> {
  iconName: IconName;
  size?: string | number;
  color?: string;
  style?: React.CSSProperties;
}

/** React Icons */
export function Icon(props: ReactIconProps) {
  const { iconName, size, color, className, style, ...rest } = props;

  return (
    <InlineSVG
      {...rest}
      src={`/icons/${iconName}.svg`}
      className={cx("slu-icon", cs.reactIcon, className)}
      style={{ height: size, color, ...(style || {}) }}
    />
  );
}

export * from "./FileIcon.tsx";
export * from "./MissingIcon.tsx";
export * from "./SpecificIcon.tsx";
