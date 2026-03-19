import { IconPackManager } from "@seelen-ui/lib";
import type { SeelenCommandGetIconArgs } from "@seelen-ui/lib/types";
import { cx } from "libs/ui/react/utils/styling.ts";
import { useComputed, useSignal, useSignalEffect } from "@preact/signals";
import { useEffect, useRef } from "react";
import type { ImgHTMLAttributes } from "react";

import { darkMode, iconPackManager } from "./common.ts";
import { MissingIcon } from "./MissingIcon.tsx";
import cs from "./index.module.css";

interface FileIconProps extends SeelenCommandGetIconArgs, Omit<ImgHTMLAttributes<HTMLImageElement>, "src"> {
  /** if true, no missing icon will be rendered in case no icon found */
  noFallback?: boolean;
}

export function FileIcon({ path, umid, noFallback, ...imgProps }: FileIconProps) {
  const $path = useSignal(path);
  const $umid = useSignal(umid);
  $path.value = path;
  $umid.value = umid;

  const icon = useComputed(() => {
    const found = iconPackManager.value.value.getIcon({ path: $path.value, umid: $umid.value });
    if (found) {
      return {
        src: (darkMode.value ? found.dark : found.light) || found.base,
        mask: found.mask,
        isAproximatelySquare: found.isAproximatelySquare,
      };
    }
    return { src: null as string | null, mask: null as string | null, isAproximatelySquare: false };
  });

  const prevSrcRef = useRef<string | null>(null);

  // On mount: always request icon extraction
  useEffect(() => {
    IconPackManager.requestIconExtraction({ path, umid });
  }, []);

  // When icon changes: if src went from non-null to null, re-request extraction
  useSignalEffect(() => {
    const src = icon.value.src;
    if (prevSrcRef.current !== null && src === null) {
      IconPackManager.requestIconExtraction({ path, umid });
    }
    prevSrcRef.current = src;
  });

  const { src, mask, isAproximatelySquare } = icon.value;

  const { ref: _ref, ...figureProps } = imgProps;
  const dataProps = Object.entries(figureProps as Record<string, unknown>)
    .filter(([k]) => k.startsWith("data-"))
    .reduce((acc, [k, v]) => ({ ...acc, [k]: v }), {} as Record<string, unknown>);

  if (src) {
    return (
      <figure
        {...figureProps}
        className={cx(cs.outer, imgProps.className)}
        data-shape={isAproximatelySquare ? "square" : "unknown"}
        data-path={path ?? undefined}
        data-umid={umid ?? undefined}
      >
        <img {...dataProps} src={src} />
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

  if (noFallback) {
    return null;
  }

  return <MissingIcon {...figureProps} />;
}
