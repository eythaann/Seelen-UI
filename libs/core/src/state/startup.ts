import { invoke, SeelenCommand } from '../handlers/mod.ts';

export class StartupManager {
  static async setAutoStart(enabled: boolean): Promise<void> {
    return await invoke(SeelenCommand.SetAutoStart, { enabled });
  }

  static async getAutoStartStatus(): Promise<boolean> {
    return await invoke(SeelenCommand.GetAutoStartStatus);
  }
}
