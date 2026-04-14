import { invoke, SeelenCommand, SeelenEvent, subscribe } from "@seelen-ui/lib";
import type { BackupStatus, UpdaterSettings } from "@seelen-ui/lib/types";
import { signal } from "@preact/signals";

import { settings } from "../../state/mod";

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

/**
 * Gets whether cloud backup sync is enabled
 */
export function getBackupSyncEnabled(): boolean {
  return settings.value.backupSyncEnabled;
}

/**
 * Enables or disables automatic cloud backup sync
 */
export function setBackupSyncEnabled(backupSyncEnabled: boolean) {
  settings.value = {
    ...settings.value,
    backupSyncEnabled,
  };
}

export const $backupStatus = signal<BackupStatus>(
  await invoke(SeelenCommand.GetBackupStatus),
);

subscribe(SeelenEvent.SeelenBackupStatusChanged, ({ payload }) => {
  $backupStatus.value = payload;
});
