import {
  getCurrentWidget,
  SeelenLauncherWidgetId,
  SeelenToolbarWidgetId,
  SeelenWallWidgetId,
  SeelenWegWidgetId,
  SeelenWindowManagerWidgetId,
  Settings,
  ThemeList,
  UIColors,
} from '@seelen-ui/lib';
import { WidgetId } from '@seelen-ui/lib/types';
import { useEffect, useState } from 'react';

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

export function isDarkModeEnabled() {
  return window.matchMedia('(prefers-color-scheme: dark)').matches;
}

export function useDarkMode() {
  const [isDarkMode, setIsDarkMode] = useState(isDarkModeEnabled());

  useEffect(() => {
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    const listener = () => setIsDarkMode(mediaQuery.matches);
    mediaQuery.addEventListener('change', listener);
    return () => mediaQuery.removeEventListener('change', listener);
  }, []);

  return isDarkMode;
}

/* backward compatibility object for old themes */
const OLD_THEME_KEYS_BY_WIDGET_ID = {
  [SeelenToolbarWidgetId]: 'toolbar',
  [SeelenWegWidgetId]: 'weg',
  [SeelenWindowManagerWidgetId]: 'wm',
  [SeelenLauncherWidgetId]: 'launcher',
  [SeelenWallWidgetId]: 'wall',
} as Record<WidgetId, string>;

function isValidCssVariableName(name: string) {
  return /^--[\w\d-]*$/.test(name);
}

function loadThemes(allThemes: ThemeList, settings: Settings) {
  const variablesByTheme = settings.inner.byTheme;
  const selected = settings.inner.selectedThemes;
  const themes = allThemes
    .asArray()
    .filter((theme) => selected.includes(theme.metadata.filename))
    .sort((a, b) => {
      return selected.indexOf(a.metadata.filename) - selected.indexOf(b.metadata.filename);
    });

  const widget = getCurrentWidget();

  document.getElementById(widget.label)?.remove();
  let element = document.createElement('style');
  element.id = widget.label;
  element.textContent = '';

  for (const theme of themes) {
    const oldKey = OLD_THEME_KEYS_BY_WIDGET_ID[widget.id];
    const cssFileContent =
      theme.styles[widget.id] || (oldKey ? theme.styles[oldKey as WidgetId] : undefined);
    if (!cssFileContent) {
      continue;
    }

    for (const def of theme.settings) {
      if (isValidCssVariableName(def.name)) {
        try {
          CSS.registerProperty({
            name: def.name,
            syntax: def.syntax,
            inherits: true,
            initialValue: `${def.initialValue}${
              'initialValueUnit' in def ? def.initialValueUnit : ''
            }`,
          });
        } catch (_e) {}
      }
    }

    const variableValues = variablesByTheme[theme.id] ?? {};
    const variablesContent = Object.entries(variableValues)
      .filter(([name]) => isValidCssVariableName(name))
      .map(([name, value]) => `${name}: ${value || ''};`)
      .join('\n');

    let layerName = 'theme-' + theme.metadata.filename.replace(/[\.]/g, '_');
    element.textContent += `@layer ${layerName} {\n:root {${variablesContent}}\n${cssFileContent}\n}\n`;
  }

  document.head.appendChild(element);
}

export async function StartThemingTool() {
  let settings = await Settings.getAsync();
  let themes = await ThemeList.getAsync();

  await ThemeList.onChange((newThemes) => {
    themes = newThemes;
    loadThemes(themes, settings);
  });

  await Settings.onChange((newSettings) => {
    settings = newSettings;
    loadThemes(themes, settings);
  });

  (await UIColors.getAsync()).setAsCssVariables();
  await UIColors.onChange((colors) => colors.setAsCssVariables());

  loadThemes(themes, settings);
}
