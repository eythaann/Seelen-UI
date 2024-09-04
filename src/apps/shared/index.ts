import { path } from '@tauri-apps/api';
import { getRootElement } from 'seelen-core';

export const getRootContainer = getRootElement;

export function toPhysicalPixels(size: number): number {
  return Math.floor(size * window.devicePixelRatio);
}

export async function wasInstalledUsingMSIX() {
  let installPath = await path.resourceDir();
  return installPath.startsWith('C:\\Program Files\\WindowsApps');
}
