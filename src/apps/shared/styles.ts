import { listen } from '@tauri-apps/api/event';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { useEffect, useState } from 'react';
import { Settings, Theme, UIColors } from 'seelen-core';

import { UserSettingsLoader } from '../settings/modules/shared/store/storeApi';

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

const KeyByLabel: Record<string, string> = {
  'fancy-toolbar': 'toolbar',
  seelenweg: 'weg',
  'window-manager': 'wm',
  'seelen-launcher': 'launcher',
  'seelen-wall': 'wall',
};

async function loadThemes(allThemes: Theme[], selected: string[]) {
  const themes = allThemes
    .filter((theme) => selected.includes(theme.info.filename))
    .sort((a, b) => {
      return selected.indexOf(a.info.filename) - selected.indexOf(b.info.filename);
    });

  const webviewId = getCurrentWebviewWindow().label;
  const [label, _monitor] = webviewId.split('/');
  if (!label) {
    return;
  }

  const theme_key = KeyByLabel[label] as keyof Theme['styles'] | undefined;
  if (!theme_key) {
    return;
  }

  document.getElementById(webviewId)?.remove();
  let element = document.createElement('style');
  element.id = webviewId;
  element.textContent = '';

  for (const theme of themes) {
    let layerName = theme.info.filename.replace(/[\.]/g, '-') + '-theme';
    element.textContent += `@layer ${layerName} {\n${theme.styles[theme_key]}\n}\n`;
  }

  document.head.appendChild(element);
}

export async function StartThemingTool() {
  const userSettings = await new UserSettingsLoader().withThemes().load();
  let allThemes = userSettings.themes;
  let selected = userSettings.jsonSettings.selectedThemes;

  await listen<Theme[]>('themes', (event) => {
    allThemes = event.payload;
    loadThemes(allThemes, selected);
  });

  await Settings.onChange((settings) => {
    selected = settings.selectedThemes;
    loadThemes(allThemes, selected);
  });

  UIColors.setAssCssVariables(await UIColors.getAsync());
  UIColors.onChange(UIColors.setAssCssVariables);

  await loadThemes(allThemes, selected);
}
