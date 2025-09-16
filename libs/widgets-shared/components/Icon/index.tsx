import { IconName } from "@shared/components/Icon/icons";
import { HTMLAttributes } from "preact/compat";

import { cx } from "../../styles";
import InlineSVG from "../InlineSvg";
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
      src={`../icons/${iconName}.svg`}
      className={cx("slu-icon", cs.reactIcon, className)}
      style={{ height: size, color, ...(style || {}) }}
    />
  );
}

export * from "./FileIcon";
export * from "./MissingIcon";
export * from "./SpecificIcon";
