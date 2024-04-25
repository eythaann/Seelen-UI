import { Theme } from '../../shared.interfaces';
import { path } from '@tauri-apps/api';
import { invoke } from '@tauri-apps/api/core';
import { PhysicalSize } from '@tauri-apps/api/dpi';
import { getCurrent } from '@tauri-apps/api/webviewWindow';

export function getRootContainer() {
  const container = document.getElementById('root');
  if (!container) {
    throw new Error('Root container not found');
  }
  return container;
}

export function toPhysicalPixels(size: number): number {
  return Math.floor(size * window.devicePixelRatio);
}

export async function wasInstalledUsingMSIX() {
  let intallPath = await path.resourceDir();
  return intallPath.startsWith('C:\\Program Files\\WindowsApps');
}

export const setWindowAsFullSize = () => {
  const screenWidth = toPhysicalPixels(window.screen.width);
  const screenHeight = toPhysicalPixels(window.screen.height);
  getCurrent().setSize(new PhysicalSize(screenWidth, screenHeight));
};

export function loadThemeCSS(theme: Theme, old?: Theme) {
  invoke<string>('get_accent_color').then((color) => {
    document.documentElement.style.setProperty('--config-accent-color', color);
  });

  if (old?.info.cssFileUrl) {
    const link = document.querySelector(`link[href="${old.info.cssFileUrl}"]`);
    if (link) {
      document.head.removeChild(link);
    }
  }

  if (theme.info.cssFileUrl) {
    const link = document.createElement('link');
    link.setAttribute('rel', 'stylesheet');
    link.setAttribute('href', theme.info.cssFileUrl);
    document.head.appendChild(link);
  }
}