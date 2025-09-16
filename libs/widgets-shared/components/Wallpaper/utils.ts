import { PlaybackSpeed, WallpaperInstanceSettings } from "@seelen-ui/lib/types";
import { CSSProperties } from "preact/compat";

export function getPlaybackRate(speed: PlaybackSpeed): number {
  switch (speed) {
    case "xDot25":
      return 0.25;
    case "xDot5":
      return 0.5;
    case "xDot75":
      return 0.75;
    case "x1":
      return 1;
    case "x1Dot25":
      return 1.25;
    case "x1Dot5":
      return 1.5;
    case "x1Dot75":
      return 1.75;
    case "x2":
      return 2;
  }
}

export function getWallpaperStyles(config: WallpaperInstanceSettings) {
  const styles: CSSProperties = {};
  const transforms: string[] = [];
  const filters: string[] = [];

  const {
    flipHorizontal,
    flipVertical,
    blur,
    saturation,
    contrast,
    objectFit,
    objectPosition,
  } = config;

  styles.objectFit = objectFit;
  styles.objectPosition = objectPosition;

  if (flipHorizontal) {
    transforms.push("scaleX(-1)");
  }

  if (flipVertical) {
    transforms.push("scaleY(-1)");
  }

  if (blur > 0) {
    filters.push(`blur(${blur}px)`);
  }

  filters.push(`saturate(${saturation})`); // 0 is allowed
  filters.push(`contrast(${contrast})`); // 0 is allowed

  if (transforms.length > 0) {
    styles.transform = transforms.join(" ");
  }

  if (filters.length > 0) {
    styles.filter = filters.join(" ");
  }

  return styles;
}
