import type { Wallpaper, WallpaperInstanceSettings } from "@seelen-ui/lib/types";

export interface BaseProps {
  definition?: Wallpaper;
  config?: WallpaperInstanceSettings;
  onLoad?: () => void;
  out?: boolean;

  muted?: boolean;
  paused?: boolean;
  pausedMessage?: string;
  /** Use the thumbnail of the wallpaper instead of the video */
  static?: boolean;
}

export interface DefinedWallProps extends BaseProps {
  definition: Wallpaper;
  config: WallpaperInstanceSettings;
}
