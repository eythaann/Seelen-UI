import { settings } from "../../state/mod";
import type { PluginId, Rect, WindowManagerSettings, WmAnimations, WmDragBehavior } from "@seelen-ui/lib/types";

/**
 * Patches the WindowManager configuration with partial updates.
 *
 * @example
 * patchWmConfig({ enabled: true });
 */
export function patchWmConfig(patch: Partial<WindowManagerSettings>) {
  settings.value = {
    ...settings.value,
    byWidget: {
      ...settings.value.byWidget,
      "@seelen/window-manager": {
        ...settings.value.byWidget["@seelen/window-manager"],
        ...patch,
      },
    },
  };
}

/**
 * Gets the current WindowManager configuration
 */
export function getWmConfig(): WindowManagerSettings {
  return settings.value.byWidget["@seelen/window-manager"];
}

/**
 * Sets the enabled state
 */
export function setWmEnabled(enabled: boolean) {
  patchWmConfig({ enabled });
}

/**
 * Sets the default layout
 */
export function setWmDefaultLayout(defaultLayout: PluginId) {
  patchWmConfig({ defaultLayout });
}

/**
 * Sets the workspace gap
 */
export function setWmWorkspaceGap(workspaceGap: number) {
  patchWmConfig({ workspaceGap });
}

/**
 * Sets the workspace padding
 */
export function setWmWorkspacePadding(workspacePadding: number) {
  patchWmConfig({ workspacePadding });
}

/**
 * Sets the workspace margin
 */
export function setWmWorkspaceMargin(workspaceMargin: Rect) {
  patchWmConfig({ workspaceMargin });
}

/**
 * Sets the resize delta
 */
export function setWmResizeDelta(resizeDelta: number) {
  patchWmConfig({ resizeDelta });
}

/**
 * Sets the drag behavior
 */
export function setWmDragBehavior(dragBehavior: WmDragBehavior) {
  patchWmConfig({ dragBehavior });
}

/**
 * Sets the animations configuration
 */
export function setWmAnimations(animations: WmAnimations) {
  patchWmConfig({ animations });
}
