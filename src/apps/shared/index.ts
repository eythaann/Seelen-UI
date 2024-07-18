import { UserSettings } from '../../shared.interfaces';
import { path } from '@tauri-apps/api';
import { invoke } from '@tauri-apps/api/core';
import { PhysicalSize } from '@tauri-apps/api/dpi';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';

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
  getCurrentWebviewWindow().setSize(new PhysicalSize(screenWidth, screenHeight));
};

export function setAccentColorAsCssVar(color: string) {
  let hex = color.replace('#', '');
  var bigint = parseInt(hex, 16);
  var r = (bigint >> 16) & 255;
  var g = (bigint >> 8) & 255;
  var b = bigint & 255;
  document.documentElement.style.setProperty('--config-accent-color', color);
  document.documentElement.style.setProperty('--config-accent-color-rgb', `${r}, ${g}, ${b}`);
}

export function loadThemeCSS(config: UserSettings) {
  invoke<string>('get_accent_color').then(setAccentColorAsCssVar);

  let selected = config.jsonSettings.selectedTheme;
  let themes = config.themes.filter((theme) => selected.includes(theme.info.filename));

  if (themes.length === 0) {
    let defaultTheme = config.themes.find((theme) => theme.info.filename === 'default');
    themes = defaultTheme ? [defaultTheme] : [];
  }

  selected.forEach((themeStr, idx) => {
    let theme = themes.find((theme) => theme.info.filename === themeStr);

    if (!theme) {
      return;
    }

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
