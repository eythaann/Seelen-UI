import { Settings, ThemeList, UIColors, Widget } from '@seelen-ui/lib';

function isValidCssVariableName(name: string) {
  return /^--[\w\d-]*$/.test(name);
}

const THEMES_STYLES_ID = 'seelen-ui-widget-themes';
const CURRENT_WIDGET_ID = Widget.getCurrentWidgetId();

async function loadThemes(allThemes: ThemeList, settings: Settings) {
  const variablesByTheme = settings.inner.byTheme;
  const activeIds = settings.inner.activeThemes;
  const themes = allThemes
    .asArray()
    .filter((theme) => activeIds.includes(theme.id))
    .sort((a, b) => activeIds.indexOf(a.id) - activeIds.indexOf(b.id));

  document.getElementById(THEMES_STYLES_ID)?.remove();
  let element = document.createElement('style');
  element.id = THEMES_STYLES_ID;
  element.textContent = '';

  for (const theme of themes) {
    const cssFileContent = theme.styles[CURRENT_WIDGET_ID];
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

    let layerName = 'theme-' + theme.metadata.path.toLowerCase().replaceAll(/[^a-zA-Z0-9]/g, '_');
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
