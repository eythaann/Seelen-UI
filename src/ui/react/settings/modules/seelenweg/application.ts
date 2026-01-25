import { settings } from "../../state/mod";
import type { SeelenWegSettings } from "@seelen-ui/lib/types";

/**
 * Patches the SeelenWeg configuration with partial updates.
 * This helper function simplifies updating the weg settings by handling
 * the nested structure automatically.
 *
 * @example
 * patchWegConfig({ enabled: true, margin: 10 });
 */
export function patchWegConfig(patch: Partial<SeelenWegSettings>) {
  settings.value = {
    ...settings.value,
    byWidget: {
      ...settings.value.byWidget,
      "@seelen/weg": {
        ...settings.value.byWidget["@seelen/weg"],
        ...patch,
      },
    },
  };
}

/**
 * Gets the current SeelenWeg configuration
 */
export function getWegConfig(): SeelenWegSettings {
  return settings.value.byWidget["@seelen/weg"];
}
