import { settings } from "../../../state/mod";
import type { ThemeId, ThemeSettings } from "@seelen-ui/lib/types";

/**
 * Gets all theme variables for a specific theme
 */
export function getThemeVariables(themeId: ThemeId): ThemeSettings {
  return settings.value.byTheme[themeId] || {};
}

/**
 * Gets a specific theme variable
 */
export function getThemeVariable(themeId: ThemeId, name: string): string | undefined {
  return settings.value.byTheme[themeId]?.[name];
}

/**
 * Sets a theme variable
 */
export function setThemeVariable(themeId: ThemeId, name: string, value: string) {
  const currentThemeVars = settings.value.byTheme[themeId] || {};

  settings.value = {
    ...settings.value,
    byTheme: {
      ...settings.value.byTheme,
      [themeId]: {
        ...currentThemeVars,
        [name]: value,
      },
    },
  };
}

/**
 * Deletes a theme variable (resets to default)
 */
export function deleteThemeVariable(themeId: ThemeId, name: string) {
  const currentThemeVars = settings.value.byTheme[themeId] || {};
  const { [name]: _, ...remainingVars } = currentThemeVars;

  settings.value = {
    ...settings.value,
    byTheme: {
      ...settings.value.byTheme,
      [themeId]: remainingVars,
    },
  };
}

/**
 * Resets all theme variables for a specific theme
 */
export function resetThemeVariables(themeId: ThemeId) {
  settings.value = {
    ...settings.value,
    byTheme: {
      ...settings.value.byTheme,
      [themeId]: {},
    },
  };
}
