import { SeelenCommand } from '@seelen-ui/lib';
import { ResourceText } from '@seelen-ui/lib/types';
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

export function getResourceText(text: ResourceText, locale: string): string {
  if (typeof text === 'string') {
    return text;
  }
  return text[locale] || text['en'] || 'Unknown';
}