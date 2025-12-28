import { useSignal } from "@preact/signals";
import { WallpaperConfiguration } from "@seelen-ui/lib";
import { WallpaperKind } from "@seelen-ui/lib/types";
import { cx } from "libs/ui/react/utils/styling.ts";
import type { ComponentChildren } from "preact";

import { ThemedWallpaper } from "./components/ThemedWallpaper";
import { ImageWallpaper } from "./components/ImageWallpaper";
import { VideoWallpaper } from "./components/VideoWallpaper";
import type { BaseProps } from "./types";
import cs from "./index.module.css";

const defaultWallpaperConfig = await WallpaperConfiguration.default();

export function Wallpaper(props: BaseProps) {
  const { definition, config = defaultWallpaperConfig } = props;

  const $loaded = useSignal(false);

  function onLoad() {
    $loaded.value = true;
    props.onLoad?.();
  }

  let element: ComponentChildren = null;

  switch (definition?.type) {
    case WallpaperKind.Image:
      element = <ImageWallpaper {...props} onLoad={onLoad} definition={definition} config={config} />;
      break;
    case WallpaperKind.Video:
      // Use thumbnail as static image for optimization when static prop is true
      if (props.static) {
        if (definition.thumbnailFilename) {
          const thumbnailDefinition = {
            ...definition,
            filename: definition.thumbnailFilename,
          };
          element = (
            <ImageWallpaper
              {...props}
              onLoad={onLoad}
              definition={thumbnailDefinition}
              config={config}
            />
          );
        }
        break;
      }

      element = <VideoWallpaper {...props} onLoad={onLoad} definition={definition} config={config} />;
      break;
    case WallpaperKind.Layered:
      element = <ThemedWallpaper {...props} onLoad={onLoad} definition={definition} config={config} />;
      break;
  }

  if (!element) {
    element = <ThemedWallpaper {...props} onLoad={onLoad} />; // Default Wallpaper
  }

  return (
    <div
      className={cx(cs.container, "wallpaper-container", {
        rendering: $loaded.value,
        "will-unrender": props.out,
      })}
    >
      {element}
      {config.withOverlay && $loaded.value && (
        <div
          className={cx(cs.overlay, "wallpaper-overlay")}
          style={{
            mixBlendMode: config.overlayMixBlendMode,
            backgroundColor: config.overlayColor,
          }}
        />
      )}
      {props.pausedMessage && props.paused && $loaded.value && definition?.type === "Video" && (
        <div className={cs.pausedMessage}>{props.pausedMessage}</div>
      )}
    </div>
  );
}
