import { cx } from "@shared/styles";
import type { HTMLAttributes } from "react";

import cs from "./infra.module.css";

interface PropsV2 extends HTMLAttributes<HTMLDivElement> {
  className?: string;
  /** for backward compatibility */
  prefix?: string;
}

export function BackgroundByLayersV2({ children, className, prefix, ...divProps }: PropsV2) {
  let background = (
    <div className={cx(cs.background, "bg-layers")} style={children ? { zIndex: -1 } : undefined}>
      {Array.from({ length: 10 }, (_, index) => (
        <div
          key={index}
          className={prefix
            ? cx(cs.layer, `bg-layer-${index + 1}`, `${prefix}-bg-layer-${index + 1}`)
            : cx(cs.layer, `bg-layer-${index + 1}`)}
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
