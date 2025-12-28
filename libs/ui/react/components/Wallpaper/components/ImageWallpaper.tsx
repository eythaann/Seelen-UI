import { cx } from "libs/ui/react/utils/styling.ts";
import { convertFileSrc } from "@tauri-apps/api/core";
import { useMemo } from "preact/hooks";

import type { DefinedWallProps } from "../types";
import { getWallpaperStyles } from "../utils.ts";
import cs from "../index.module.css";

export function ImageWallpaper({ definition, config, onLoad }: DefinedWallProps) {
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
  };

  return (
    <img
      id={definition.id}
      className={cx(cs.wallpaper, "wallpaper")}
      style={getWallpaperStyles(config)}
      src={imageSrc}
      onLoad={onLoad}
      onError={handleError}
      decoding="async"
      loading="eager"
    />
  );
}
