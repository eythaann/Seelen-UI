import { invoke } from '@tauri-apps/api/core';
import * as oldStartup from '@tauri-apps/plugin-autostart';
import { SeelenCommand } from 'seelen-core';

export * as dialog from '@tauri-apps/plugin-dialog';
export * as fs from '@tauri-apps/plugin-fs';

export class startup {
  static async enable(): Promise<void> {
    await invoke(SeelenCommand.SetAutoStart, { enabled: true });
  }

  static async disable(): Promise<void> {
    await invoke(SeelenCommand.SetAutoStart, { enabled: false });
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