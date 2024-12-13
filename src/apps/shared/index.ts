import { invoke } from '@tauri-apps/api/core';
import { getRootElement, SeelenCommand } from 'seelen-core';

export const getRootContainer = getRootElement;

export function toPhysicalPixels(size: number): number {
  return Math.round(size * window.devicePixelRatio);
}

export async function wasInstalledUsingMSIX(): Promise<boolean> {
  // Todo replace this when added to SeelenCommand
  return invoke('is_appx_package');
}

export async function isDev(): Promise<boolean> {
  return invoke(SeelenCommand.IsDevMode);
}
