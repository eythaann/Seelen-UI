import { cx } from "libs/ui/react/utils/styling.ts";
import { useComputed } from "@preact/signals";
import type { ImgHTMLAttributes } from "react";

import { darkMode, iconPackManager } from "./common.ts";
import cs from "./index.module.css";

interface MissingIconProps extends Omit<ImgHTMLAttributes<HTMLImageElement>, "src"> {}

export function MissingIcon({ ref: _ref, ...props }: MissingIconProps) {
  const icon = useComputed(() => {
    const found = iconPackManager.value.value.getMissingIcon();
    if (found) {
      return {
        src: (darkMode.value ? found.dark : found.light) || found.base,
        mask: found.mask,
      };
    }
    return { src: null as string | null, mask: null as string | null };
  });

  const { src, mask } = icon.value;

  const dataProps = Object.entries(props as Record<string, unknown>)
    .filter(([k]) => k.startsWith("data-"))
    .reduce((acc, [k, v]) => ({ ...acc, [k]: v }), {} as Record<string, unknown>);

  return (
    <figure {...props} className={cx(cs.outer, props.className)}>
      <img {...dataProps} src={src || ""} />
      {mask && (
        <div
          {...dataProps}
          className={cs.mask}
          style={{ maskImage: `url('${mask}')` }}
        />
      )}
    </figure>
  );
}
