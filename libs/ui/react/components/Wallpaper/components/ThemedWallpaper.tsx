import { cx } from "libs/ui/react/utils/styling.ts";
import { useEffect } from "preact/hooks";
import { BackgroundByLayersV2 } from "libs/ui/react/components/BackgroundByLayers/infra.tsx";

import type { BaseProps } from "../types";
import { getWallpaperStyles } from "../utils.ts";
import cs from "../index.module.css";

export function ThemedWallpaper({
  definition,
  config,
  onLoad,
}: Pick<BaseProps, "definition" | "config" | "onLoad">) {
  useEffect(() => {
    onLoad?.();
  }, []);

  if (!definition || !config) {
    return (
      <div className={cx(cs.wallpaper, cs.defaultWallpaper)}>
        <BackgroundByLayersV2 />
      </div>
    );
  }

  return (
    <div id={definition.id} className={cs.wallpaper} style={getWallpaperStyles(config)}>
      <style>{`@scope { ${definition.css || ""} }`}</style>
      <BackgroundByLayersV2 />
    </div>
  );
}
