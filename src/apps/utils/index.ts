import { UserSettings } from '../../shared.interfaces';
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
  let installPath = await path.resourceDir();
  return installPath.startsWith('C:\\Program Files\\WindowsApps');
}

export const setWindowAsFullSize = () => {
  const screenWidth = toPhysicalPixels(window.screen.width);
  const screenHeight = toPhysicalPixels(window.screen.height);
  getCurrent().setSize(new PhysicalSize(screenWidth, screenHeight));
};

export function loadThemeCSS(config: UserSettings) {
  invoke<string>('get_accent_color').then((color) => {
    document.documentElement.style.setProperty('--config-accent-color', color);
  });

  let selected = [config.jsonSettings.selectedTheme || ''].flat();
  let themes = config.themes.filter((theme) => selected.includes(theme.info.filename));

  if (themes.length === 0) {
    let defaultTheme = config.themes.find((theme) => theme.info.filename === 'default');
    themes = defaultTheme ? [defaultTheme] : [];
  }

  themes.forEach((theme, idx) => {
    for (const key of Object.keys(theme.styles)) {
      let element = document.getElementById(key);

      if (!element) {
        element = document.createElement('style');
        element.id = key.toString();
        document.head.appendChild(element);
      }

      if (idx === 0) {
        element.textContent = '';
      }

      element.textContent += theme.styles[key] + '\n';
    }
  });
}
