import { settings } from "../../../state/mod";
import type { WallpaperId, WallpaperInstanceSettings } from "@seelen-ui/lib/types";

/**
 * Gets the settings for a specific wallpaper
 */
export function getWallpaperSettings(id: WallpaperId): WallpaperInstanceSettings | undefined {
  return settings.value.byWallpaper[id];
}

/**
 * Patches the settings for a specific wallpaper
 */
export function patchWallpaperSettings(id: WallpaperId, patch: Partial<WallpaperInstanceSettings>) {
  settings.value = {
    ...settings.value,
    byWallpaper: {
      ...settings.value.byWallpaper,
      [id]: {
        ...(settings.value.byWallpaper[id] || {}),
        ...patch,
      } as WallpaperInstanceSettings,
    },
  };
}

/**
 * Resets the settings for a specific wallpaper
 */
export function resetWallpaperSettings(id: WallpaperId) {
  const { [id]: _, ...rest } = settings.value.byWallpaper;
  settings.value = {
    ...settings.value,
    byWallpaper: rest,
  };
}
