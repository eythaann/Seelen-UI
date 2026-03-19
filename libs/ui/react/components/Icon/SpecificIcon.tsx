import { cx } from "libs/ui/react/utils/styling.ts";
import { useComputed, useSignal } from "@preact/signals";
import type { ImgHTMLAttributes } from "react";

import { darkMode, iconPackManager } from "./common.ts";
import cs from "./index.module.css";

interface SpecificIconProps extends Omit<ImgHTMLAttributes<HTMLImageElement>, "src"> {
  name: string;
}

export function SpecificIcon({ name, ref: _ref, ...imgProps }: SpecificIconProps) {
  const $name = useSignal(name);
  $name.value = name;

  const icon = useComputed(() => {
    const found = iconPackManager.value.value.getCustomIcon($name.value);
    if (found) {
      return {
        src: (darkMode.value ? found.dark : found.light) || found.base,
        mask: found.mask,
        isAproximatelySquare: found.isAproximatelySquare,
      };
    }
    return { src: null as string | null, mask: null as string | null, isAproximatelySquare: false };
  });

  const { src, mask, isAproximatelySquare } = icon.value;

  if (!src) {
    return null;
  }

  return (
    <figure
      {...imgProps}
      className={cx(cs.outer, imgProps.className)}
      data-shape={isAproximatelySquare ? "square" : "unknown"}
    >
      <img src={src} />
      {mask && (
        <div
          className={cs.mask}
          style={{ maskImage: `url('${mask}')` }}
        />
      )}
    </figure>
  );
}
