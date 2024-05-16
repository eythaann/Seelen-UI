import { invoke } from '@tauri-apps/api/core';
import * as oldStartup from '@tauri-apps/plugin-autostart';

export * as dialog from '@tauri-apps/plugin-dialog';
export * as fs from '@tauri-apps/plugin-fs';

export class startup {
  static async enable(): Promise<void> {
    await invoke('set_auto_start', { enabled: true });
  }

  static async disable(): Promise<void> {
    await invoke('set_auto_start', { enabled: false });
  }

  static async isEnabled(): Promise<boolean> {
    // migration from old autostart to new
    if (await oldStartup.isEnabled()) {
      await oldStartup.disable();
      await startup.enable();
    }
    return await invoke<boolean>('get_auto_start_status');
  }
}