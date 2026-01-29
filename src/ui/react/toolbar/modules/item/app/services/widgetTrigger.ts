import { Alignment, FancyToolbarSide, type WidgetId } from "@seelen-ui/lib/types";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { toPhysicalPixels } from "libs/ui/react/utils/index.ts";
import { invoke, SeelenCommand } from "@seelen-ui/lib";
import { $settings } from "../../../shared/state/mod.ts";

/**
 * Triggers a widget at a calculated position relative to the toolbar item.
 * @param widgetId - The ID of the widget to trigger
 * @param itemId - The ID of the toolbar item element
 */
export async function triggerWidget(widgetId: WidgetId, itemId: string): Promise<void> {
  if (typeof widgetId !== "string") {
    return;
  }

  const { x: windowX, y: windowY } = await getCurrentWindow().outerPosition();

  // Get position of the element on the screen
  const element = document.getElementById(itemId);
  if (!element) {
    console.error(`Element with id ${itemId} not found`);
    return;
  }

  const domRect = element.getBoundingClientRect();
  const x = windowX + toPhysicalPixels(domRect.left + domRect.width / 2);

  const rootRect = document.getElementById("root")!.getBoundingClientRect();
  const isTopPosition = $settings.value.position === FancyToolbarSide.Top;

  const y = isTopPosition ? windowY + toPhysicalPixels(rootRect.bottom) : windowY + toPhysicalPixels(rootRect.top);

  await invoke(SeelenCommand.TriggerWidget, {
    payload: {
      id: widgetId,
      desiredPosition: { x, y },
      alignX: Alignment.Center,
      alignY: isTopPosition ? Alignment.End : Alignment.Start,
    },
  });
}
