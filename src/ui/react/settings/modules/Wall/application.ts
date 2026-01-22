import { settings } from "../../state/mod";
import type {
  SeelenWallSettings,
  WallpaperCollection,
  WallpaperId,
  WallpaperInstanceSettings,
} from "@seelen-ui/lib/types";

/**
 * Patches the Wall configuration with partial updates.
 *
 * @example
 * patchWallConfig({ enabled: true, interval: 3600 });
 */
export function patchWallConfig(patch: Partial<SeelenWallSettings>) {
  settings.value = {
    ...settings.value,
    byWidget: {
      ...settings.value.byWidget,
      "@seelen/wallpaper-manager": {
        ...settings.value.byWidget["@seelen/wallpaper-manager"],
        ...patch,
      },
    },
  };
}

/**
 * Gets the current Wall configuration
 */
export function getWallConfig(): SeelenWallSettings {
  return settings.value.byWidget["@seelen/wallpaper-manager"];
}

/**
 * Gets all wallpaper collections
 */
export function getWallpaperCollections(): WallpaperCollection[] {
  return settings.value.wallpaperCollections;
}

/**
 * Adds a new wallpaper collection
 */
export function addWallpaperCollection(collection: WallpaperCollection) {
  settings.value = {
    ...settings.value,
    wallpaperCollections: [...settings.value.wallpaperCollections, collection],
  };
}

/**
 * Updates an existing wallpaper collection
 */
export function updateWallpaperCollection(collection: WallpaperCollection) {
  const index = settings.value.wallpaperCollections.findIndex((c) => c.id === collection.id);
  if (index === -1) return;

  const newCollections = [...settings.value.wallpaperCollections];
  newCollections[index] = collection;

  settings.value = {
    ...settings.value,
    wallpaperCollections: newCollections,
  };
}

/**
 * Deletes a wallpaper collection and cleans up references
 */
export function deleteWallpaperCollection(collectionId: string) {
  const newCollections = settings.value.wallpaperCollections.filter((c) => c.id !== collectionId);

  // Reset default collection if it was deleted
  const newDefaultCollection = settings.value.byWidget["@seelen/wallpaper-manager"].defaultCollection === collectionId
    ? null
    : settings.value.byWidget["@seelen/wallpaper-manager"].defaultCollection;

  // Reset monitor collections if they were using this collection
  const newMonitors = { ...settings.value.monitorsV3 };
  Object.keys(newMonitors).forEach((monitorId) => {
    const monitor = newMonitors[monitorId];
    if (monitor && monitor.wallpaperCollection === collectionId) {
      newMonitors[monitorId] = {
        ...monitor,
        wallpaperCollection: null,
      };
    }
  });

  settings.value = {
    ...settings.value,
    wallpaperCollections: newCollections,
    monitorsV3: newMonitors,
    byWidget: {
      ...settings.value.byWidget,
      "@seelen/wallpaper-manager": {
        ...settings.value.byWidget["@seelen/wallpaper-manager"],
        defaultCollection: newDefaultCollection,
      },
    },
  };
}

/**
 * Sets the default wallpaper collection
 */
export function setDefaultWallpaperCollection(collectionId: string | null) {
  patchWallConfig({ defaultCollection: collectionId });
}

/**
 * Sets a wallpaper collection for a specific monitor
 */
export function setMonitorWallpaperCollection(monitorId: string, collectionId: string | null) {
  const monitor = settings.value.monitorsV3[monitorId];
  if (!monitor) return;

  settings.value = {
    ...settings.value,
    monitorsV3: {
      ...settings.value.monitorsV3,
      [monitorId]: {
        ...monitor,
        wallpaperCollection: collectionId,
      },
    },
  };
}

/**
 * Sets a wallpaper collection for a specific workspace on a monitor
 */
export function setWorkspaceWallpaperCollection(
  monitorId: string,
  workspaceId: string,
  collectionId: string | null,
) {
  const monitor = settings.value.monitorsV3[monitorId];
  if (!monitor) return;

  settings.value = {
    ...settings.value,
    monitorsV3: {
      ...settings.value.monitorsV3,
      [monitorId]: {
        ...monitor,
        byWorkspace: {
          ...(monitor.byWorkspace || {}),
          [workspaceId]: {
            ...(monitor.byWorkspace?.[workspaceId] || {}),
            wallpaperCollection: collectionId,
          },
        },
      },
    },
  };
}

/**
 * Patches settings for a specific wallpaper instance
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
 * Resets settings for a specific wallpaper instance
 */
export function resetWallpaperSettings(id: WallpaperId) {
  const newByWallpaper = { ...settings.value.byWallpaper };
  delete newByWallpaper[id];

  settings.value = {
    ...settings.value,
    byWallpaper: newByWallpaper,
  };
}
