import { resolve, resourceDir } from "@tauri-apps/api/path";

export const DEFAULT_THUMBNAIL = await resolve(
  await resourceDir(),
  "static",
  "icons",
  "music_thumbnail.jpg",
);
