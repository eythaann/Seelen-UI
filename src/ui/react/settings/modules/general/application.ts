import { needRestart, settings } from "../../state/mod";
import type { PerformanceModeSettings, StartOfWeek } from "@seelen-ui/lib/types";

/**
 * Gets the current language setting
 */
export function getLanguage(): string | null {
  return settings.value.language;
}

/**
 * Sets the language
 */
export function setLanguage(language: string | null) {
  settings.value = {
    ...settings.value,
    language,
  };
}

/**
 * Gets the date format
 */
export function getDateFormat(): string {
  return settings.value.dateFormat;
}

/**
 * Sets the date format
 */
export function setDateFormat(dateFormat: string) {
  settings.value = {
    ...settings.value,
    dateFormat,
  };
}

/**
 * Gets the start of week setting
 */
export function getStartOfWeek(): StartOfWeek {
  return settings.value.startOfWeek;
}

/**
 * Sets the start of week
 */
export function setStartOfWeek(startOfWeek: StartOfWeek) {
  settings.value = {
    ...settings.value,
    startOfWeek,
  };
}

/**
 * Gets the performance mode settings
 */
export function getPerformanceMode(): PerformanceModeSettings {
  return settings.value.performanceMode;
}

/**
 * Sets the performance mode settings
 */
export function setPerformanceMode(performanceMode: PerformanceModeSettings) {
  settings.value = {
    ...settings.value,
    performanceMode,
  };
}

/**
 * Gets the hardware acceleration setting
 */
export function getHardwareAcceleration(): boolean {
  return settings.value.hardwareAcceleration;
}

/**
 * Sets the hardware acceleration setting
 */
export function setHardwareAcceleration(hardwareAcceleration: boolean) {
  needRestart.value = true;
  settings.value = {
    ...settings.value,
    hardwareAcceleration,
  };
}

/**
 * Patches the performance mode settings with partial updates
 */
export function patchPerformanceMode(patch: Partial<PerformanceModeSettings>) {
  settings.value = {
    ...settings.value,
    performanceMode: {
      ...settings.value.performanceMode,
      ...patch,
    },
  };
}
