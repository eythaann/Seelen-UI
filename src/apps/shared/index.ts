import { path } from '@tauri-apps/api';
import { getRootElement, invoke, SeelenCommand } from 'seelen-core';

export const getRootContainer = getRootElement;

export function toPhysicalPixels(size: number): number {
  return Math.round(size * window.devicePixelRatio);
}

export async function wasInstalledUsingMSIX() {
  let installPath = await path.resourceDir();
  return installPath.startsWith('C:\\Program Files\\WindowsApps');
}

export async function isDev() {
  return invoke(SeelenCommand.IsDevMode);
}