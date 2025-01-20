import { SeelenCommand } from '@seelen-ui/lib';
import { invoke } from '@tauri-apps/api/core';

export function getRootContainer(): HTMLElement {
  const element = document.getElementById('root');
  if (!element) {
    throw new Error('Root element not found');
  }
  return element;
}

export function toPhysicalPixels(size: number): number {
  return Math.round(size * window.devicePixelRatio);
}

export async function wasInstalledUsingMSIX(): Promise<boolean> {
  return invoke(SeelenCommand.IsAppxPackage);
}

export async function isDev(): Promise<boolean> {
  return invoke(SeelenCommand.IsDevMode);
}
