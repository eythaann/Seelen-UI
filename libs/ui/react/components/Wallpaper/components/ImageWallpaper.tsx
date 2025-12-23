import { cx } from "libs/ui/react/utils/styling.ts";
import { convertFileSrc } from "@tauri-apps/api/core";
import { useMemo, useRef } from "preact/hooks";

import type { DefinedWallProps } from "../types";
import { getWallpaperStyles } from "../utils.ts";
import cs from "../index.module.css";

const MAX_RETRIES = 2;

export function ImageWallpaper({ definition, config, onLoad }: DefinedWallProps) {
  const retryCountRef = useRef(0);

  const imageSrc = useMemo(
    () => convertFileSrc(definition.metadata.path + "\\" + definition.filename!),
    [definition.metadata.path, definition.filename],
  );

  const handleError = (e: Event) => {
    const target = e.target as HTMLImageElement;
    console.error("Image failed to load:", {
      src: imageSrc,
      naturalWidth: target.naturalWidth,
      naturalHeight: target.naturalHeight,
    });

    // Attempt retry for network issues
    if (retryCountRef.current < MAX_RETRIES) {
      retryCountRef.current++;
      console.debug(`Retrying image load (${retryCountRef.current}/${MAX_RETRIES})`);

      // Force reload by adding timestamp
      setTimeout(() => {
        const timestamp = Date.now();
        target.src = `${imageSrc}?retry=${timestamp}`;
      }, 1000);
    }
  };

  const handleLoad = () => {
    retryCountRef.current = 0; // Reset on successful load
    onLoad?.();
  };

  return (
    <img
      id={definition.id}
      className={cx(cs.wallpaper, "wallpaper")}
      style={getWallpaperStyles(config)}
      src={imageSrc}
      onLoad={handleLoad}
      onError={handleError}
      decoding="async"
      loading="eager"
    />
  );
}
