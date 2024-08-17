import { UserSettings } from '../../shared.interfaces';
import { Theme } from './schemas/Theme';
import { path } from '@tauri-apps/api';
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

export function setColorsAsCssVariables(colors: anyObject) {
  for (const [key, value] of Object.entries(colors)) {
    if (typeof value !== 'string') {
      continue;
    }
    let hex = value.replace('#', '').slice(0, 6);
    var color = parseInt(hex, 16);
    var r = (color >> 16) & 255;
    var g = (color >> 8) & 255;
    var b = color & 255;
    // replace rust snake case with kebab case
    let name = key.replace('_', '-');
    document.documentElement.style.setProperty(`--config-${name}-color`, value.slice(0, 7));
    document.documentElement.style.setProperty(`--config-${name}-color-rgb`, `${r}, ${g}, ${b}`);
  }
}

export function loadThemeCSS(config: Pick<UserSettings, 'themes' | 'jsonSettings'>) {
  let selected = config.jsonSettings.selectedTheme;
  let themes: Theme[] = config.themes
    .filter((theme) => selected.includes(theme.info.filename))
    .sort((a, b) => {
      return selected.indexOf(a.info.filename) - selected.indexOf(b.info.filename);
    });

  if (themes.length === 0) {
    let defaultTheme = config.themes.find((theme) => theme.info.filename === 'default');
    themes = defaultTheme ? [defaultTheme] : [];
  }

  const label = getCurrentWebviewWindow().label;
  let theme_key: keyof Theme['styles'] | null = null;
  if (label.startsWith('fancy-toolbar')) {
    theme_key = 'toolbar';
  } else if (label.startsWith('seelenweg')) {
    theme_key = 'weg';
  } else if (label.startsWith('window-manager')) {
    theme_key = 'wm';
  }

  if (!theme_key) {
    return;
  }

  document.getElementById(theme_key)?.remove();
  let element = document.createElement('style');
  element.id = theme_key.toString();
  element.textContent = '';
  document.head.appendChild(element);
  for (const theme of themes) {
    element.textContent += theme.styles[theme_key] + '\n';
  }
}
