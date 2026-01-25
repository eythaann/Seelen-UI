import { settings } from "../../state/mod";
import type { FancyToolbarSettings, FancyToolbarSide, HideMode } from "@seelen-ui/lib/types";

/**
 * Patches the FancyToolbar configuration with partial updates.
 * This helper function simplifies updating the toolbar settings by handling
 * the nested structure automatically.
 *
 * @example
 * patchToolbarConfig({ enabled: true, height: 40 });
 */
export function patchToolbarConfig(patch: Partial<FancyToolbarSettings>) {
  settings.value = {
    ...settings.value,
    byWidget: {
      ...settings.value.byWidget,
      "@seelen/fancy-toolbar": {
        ...settings.value.byWidget["@seelen/fancy-toolbar"],
        ...patch,
      },
    },
  };
}

/**
 * Gets the current FancyToolbar configuration
 */
export function getToolbarConfig(): FancyToolbarSettings {
  return settings.value.byWidget["@seelen/fancy-toolbar"];
}

/**
 * Sets the enabled state
 */
export function setToolbarEnabled(enabled: boolean) {
  patchToolbarConfig({ enabled });
}

/**
 * Sets the toolbar height
 */
export function setToolbarHeight(height: number) {
  patchToolbarConfig({ height });
}

/**
 * Sets the toolbar position
 */
export function setToolbarPosition(position: FancyToolbarSide) {
  patchToolbarConfig({ position });
}

/**
 * Sets the hide mode
 */
export function setToolbarHideMode(hideMode: HideMode) {
  patchToolbarConfig({ hideMode });
}

/**
 * Sets the delay to show
 */
export function setToolbarDelayToShow(delayToShow: number) {
  patchToolbarConfig({ delayToShow });
}

/**
 * Sets the delay to hide
 */
export function setToolbarDelayToHide(delayToHide: number) {
  patchToolbarConfig({ delayToHide });
}
