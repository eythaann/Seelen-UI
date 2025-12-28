import { UIColors } from "../../system_state/ui_colors.ts";
import { RuntimeStyleSheet } from "../../utils/DOM.ts";
import { Settings } from "../settings/mod.ts";
import { ThemeList } from "./mod.ts";

/**
 * This will apply the active themes for this widget, and automatically update
 * when the themes or settings change. Also will add the systehm ui colors to the document.
 */
export async function startThemingTool(): Promise<void> {
  let settings = await Settings.getAsync();
  let themes = await ThemeList.getAsync();

  await ThemeList.onChange((newThemes) => {
    themes = newThemes;
    themes.applyToDocument(settings.activeThemes, settings.byTheme);
  });

  await Settings.onChange((newSettings) => {
    settings = newSettings;
    themes.applyToDocument(settings.activeThemes, settings.byTheme);
  });

  (await UIColors.getAsync()).setAsCssVariables();
  await UIColors.onChange((colors) => colors.setAsCssVariables());

  startDateCssVariables();

  themes.applyToDocument(settings.activeThemes, settings.byTheme);
}

export function startDateCssVariables(): void {
  // Set initial values immediately
  updateDateCssVariables();
  // Update every minute (60000ms) to avoid overhead from seconds
  setInterval(updateDateCssVariables, 60000);
}

function updateDateCssVariables(): void {
  const now = new Date();
  const locale = navigator.language;

  // Time values
  const hour = now.getHours(); // 0-23
  const minute = now.getMinutes(); // 0-59

  // Date name values using Intl API
  const dayName = new Intl.DateTimeFormat(locale, { weekday: "long" }).format(now);
  const monthName = new Intl.DateTimeFormat(locale, { month: "long" }).format(now);

  // Date numeric values
  const dayOfMonth = now.getDate(); // 1-31
  const monthNumber = now.getMonth() + 1; // 1-12
  const year = now.getFullYear(); // 2025, etc.

  const styleSheet = new RuntimeStyleSheet("@runtime/date-variables");

  // Time variables
  styleSheet.addVariable("--date-hour", String(hour));
  styleSheet.addVariable("--date-minute", String(minute));

  // Date name variables (localized)
  styleSheet.addVariable("--date-day-name", dayName);
  styleSheet.addVariable("--date-month-name", monthName);

  // Date numeric variables
  styleSheet.addVariable("--date-day", String(dayOfMonth));
  styleSheet.addVariable("--date-month", String(monthNumber));
  styleSheet.addVariable("--date-year", String(year));

  styleSheet.applyToDocument();
}
