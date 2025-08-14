import { SeelenCommand } from '@seelen-ui/lib';
import { invoke } from '@tauri-apps/api/core';

export class startup {
  static async enable(): Promise<void> {
    await invoke(SeelenCommand.SetAutoStart, { enabled: true });
  }

  static async disable(): Promise<void> {
    await invoke(SeelenCommand.SetAutoStart, { enabled: false });
  }

  static async isEnabled(): Promise<boolean> {
    return await invoke<boolean>('get_auto_start_status');
  }
}
