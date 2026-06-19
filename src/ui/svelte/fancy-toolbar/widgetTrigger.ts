import { Alignment, FancyToolbarSide, type WidgetId } from "@seelen-ui/lib/types";
import { toPhysicalPixels } from "libs/ui/react/utils/index.ts";
import { invoke, SeelenCommand } from "@seelen-ui/lib";
import { settingsState, widgetRect } from "./state/settings.svelte.ts";

export function triggerWidget(widgetId: WidgetId, itemId: string): void {
  if (typeof widgetId !== "string") {
    return;
  }

  const { left: windowX, top: windowY } = widgetRect.value;

  const element = document.getElementById(itemId);
  if (!element) {
    console.error(`Element with id ${itemId} not found`);
    return;
  }

  const domRect = element.getBoundingClientRect();
  const x = windowX + toPhysicalPixels(domRect.left + domRect.width / 2);

  const rootRect = document.getElementById("root")!.getBoundingClientRect();
  const isTopPosition = settingsState.position === FancyToolbarSide.Top;

  const y = isTopPosition ? windowY + toPhysicalPixels(rootRect.bottom) : windowY + toPhysicalPixels(rootRect.top);

  invoke(SeelenCommand.TriggerWidget, {
    payload: {
      id: widgetId,
      desiredPosition: { x, y },
      alignX: Alignment.Center,
      alignY: isTopPosition ? Alignment.Start : Alignment.End,
    },
  });
}
