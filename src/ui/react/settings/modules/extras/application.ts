import { settings } from "../../state/mod";
import type { UpdaterSettings } from "@seelen-ui/lib/types";

/**
 * Gets the Discord RPC setting
 */
export function getDrpc(): boolean {
  return settings.value.drpc;
}

/**
 * Gets the streaming mode setting
 */
export function getStreamingMode(): boolean {
  return settings.value.streamingMode;
}

/**
 * Sets the streaming mode setting
 */
export function setStreamingMode(streamingMode: boolean) {
  settings.value = {
    ...settings.value,
    streamingMode,
  };
}

/**
 * Sets the Discord RPC setting
 */
export function setDrpc(drpc: boolean) {
  settings.value = {
    ...settings.value,
    drpc,
  };
}

/**
 * Gets the updater settings
 */
export function getUpdaterSettings(): UpdaterSettings {
  return settings.value.updater;
}

/**
 * Patches the updater settings with partial updates
 */
export function patchUpdaterSettings(patch: Partial<UpdaterSettings>) {
  settings.value = {
    ...settings.value,
    updater: {
      ...settings.value.updater,
      ...patch,
    },
  };
}
