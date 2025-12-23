import { invoke, SeelenCommand } from "@seelen-ui/lib";
import type { Wallpaper } from "@seelen-ui/lib/types";
import { WallpaperKind } from "@seelen-ui/lib/types";
import { convertFileSrc } from "@tauri-apps/api/core";

import { extractThumbnailFromSource } from "./videoThumbnail.ts";

export interface ThumbGenerationProgress {
  current: number;
  total: number;
  currentVideoName: string | null;
}

export type ProgressCallback = (progress: ThumbGenerationProgress) => void;

/**
 * Gets a display name from ResourceText
 */
function getDisplayName(wallpaper: Wallpaper): string {
  const displayName = wallpaper.metadata.displayName;
  if (typeof displayName === "string") {
    return displayName;
  }
  // If it's an object, try to get the English translation or first available
  return displayName.en || Object.values(displayName)[0] || wallpaper.filename || "Unknown";
}

/**
 * Filters video wallpapers that don't have a thumbnail
 */
export function getVideosWithoutThumbnail(wallpapers: Wallpaper[]): Wallpaper[] {
  return wallpapers.filter(
    (wallpaper) =>
      wallpaper.type === WallpaperKind.Video &&
      !wallpaper.thumbnailFilename &&
      !wallpaper.thumbnailUrl &&
      wallpaper.filename, // Only process if it has a local file
  );
}

/**
 * Generates thumbnails for video wallpapers one by one
 * @param wallpapers - List of wallpapers to process (should be video wallpapers without thumbnails)
 * @param onProgress - Callback to report progress
 * @returns Promise that resolves when all thumbnails are generated
 */
export async function generateThumbnails(
  wallpapers: Wallpaper[],
  onProgress: ProgressCallback,
): Promise<void> {
  const total = wallpapers.length;

  for (let i = 0; i < wallpapers.length; i++) {
    const wallpaper = wallpapers[i];
    if (!wallpaper) {
      continue;
    }

    onProgress({
      current: i,
      total,
      currentVideoName: getDisplayName(wallpaper),
    });

    try {
      if (!wallpaper.filename) {
        console.warn(`Wallpaper ${wallpaper.id} has no filename, skipping`);
        continue;
      }

      const videoSrc = convertFileSrc(wallpaper.metadata.path + "\\" + wallpaper.filename);
      const thumbnailBytes = await extractThumbnailFromSource(videoSrc, 0.9);
      if (!thumbnailBytes) {
        console.error(`Failed to extract thumbnail for ${wallpaper.id}`);
        continue;
      }

      await invoke(SeelenCommand.WallpaperSaveThumbnail, {
        wallpaperId: wallpaper.id,
        thumbnailBytes: Array.from(thumbnailBytes),
      });
    } catch (error) {
      console.error(`Error generating thumbnail for ${wallpaper.id}:`, error);
    }
  }

  onProgress({
    current: total,
    total,
    currentVideoName: null,
  });
}
