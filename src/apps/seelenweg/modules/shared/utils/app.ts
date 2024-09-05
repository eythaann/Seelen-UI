import { path } from '@tauri-apps/api';

export function filenameFromPath(path: string): string {
  const parts = path.split('\\');
  return parts[parts.length - 1] || '';
}

export async function getGeneratedFilesPath(): Promise<string> {
  return await path.appDataDir();
}
