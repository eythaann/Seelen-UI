import { UserSettingsLoader } from '../settings/modules/shared/store/storeApi';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { useEffect, useState } from 'react';
import { EventHandler, Settings, Theme } from 'seelen-core';

type Args = undefined | string | { [x: string]: any };
export const cx = (...args: Args[]): string => {
  return args
    .map((arg) => {
      if (!arg) {
        return;
      }

      if (typeof arg === 'string') {
        return arg;
      }

      let classnames = '';
      Object.keys(arg).forEach((key) => {
        if (arg[key]) {
          classnames += ` ${key}`;
        }
      });

      return classnames.trimStart();
    })
    .join(' ');
};

export function useDarkMode() {
  const [isDarkMode, setIsDarkMode] = useState(
    window.matchMedia('(prefers-color-scheme: dark)').matches,
  );

  useEffect(() => {
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    const listener = () => setIsDarkMode(mediaQuery.matches);
    mediaQuery.addEventListener('change', listener);
    return () => mediaQuery.removeEventListener('change', listener);
  });

  return isDarkMode;
}

async function loadThemes(allThemes: Theme[], selected: string[]) {
  let themes = allThemes
    .filter((theme) => selected.includes(theme.info.filename))
    .sort((a, b) => {
      return selected.indexOf(a.info.filename) - selected.indexOf(b.info.filename);
    });

  if (themes.length === 0) {
    let defaultTheme = themes.find((theme) => theme.info.filename === 'default');
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
  } else if (label.startsWith('seelen-launcher')) {
    theme_key = 'launcher';
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

export async function StartThemingTool() {
  const userSettings = await new UserSettingsLoader().withThemes().load();
  let allThemes = userSettings.themes;
  let selected = userSettings.jsonSettings.selectedTheme;

  await listen<Theme[]>('themes', (event) => {
    allThemes = event.payload;
    loadThemes(allThemes, selected);
  });

  await listen<Settings>(EventHandler.Settings, (event) => {
    selected = event.payload.selectedTheme;
    loadThemes(allThemes, selected);
  });

  await loadThemes(allThemes, selected);
}
