import { path } from '@tauri-apps/api';

export function toPhysicalPixels(size: number): number {
  return Math.floor(size * window.devicePixelRatio);
}

export async function wasInstalledUsingMSIX() {
  let intallPath = await path.resourceDir();
  return intallPath.startsWith('C:\\Program Files\\WindowsApps');
}