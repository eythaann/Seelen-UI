import { path } from '@tauri-apps/api';

export function filenameFromPath(path: string): string {
  const parts = path.split('\\');
  return parts[parts.length - 1] || '';
}

export async function getGeneratedFilesPath(): Promise<string> {
  return await path.appDataDir();
}

export class CallbacksManager {
  callbacks: Record<string, () => void> = {};

  add(cb: () => void, key: string) {
    this.callbacks[key] = cb;
  }

  remove(key: string) {
    delete this.callbacks[key];
  }

  execute() {
    Object.values(this.callbacks).forEach((cb) => cb());
  }
}