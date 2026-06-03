import { invoke, SeelenCommand, Widget } from "@seelen-ui/lib";
import type { Rect } from "@seelen-ui/lib/types";
import { toPhysicalPixels } from "libs/ui/react/utils";
import type { State } from "../state.svelte";

const monitorId = Widget.self.decoded.monitorId!;

export function requestPositioningOfLeaves(state: State) {
  const someIsMaximizedOnBg = state.interactables.some(
    (app) => app.monitor === monitorId && (app.isZoomed || app.isFullscreen) && !app.isIconic,
  );

  if (someIsMaximizedOnBg || state.paused) {
    return;
  }

  const { left: windowX, top: windowY } = state.widgetRect;

  let elements = document.querySelectorAll("[data-hwnd]");
  let positions: Record<string, Rect> = {};

  // Svelte 5 runes: direct access to state
  const borderConfig = state.settings.border;
  elements.forEach((element) => {
    let hwnd = (element as HTMLDivElement).dataset.hwnd!;
    const border = borderConfig.enabled ? borderConfig.width + borderConfig.offset : 0;

    const domRect = element.getBoundingClientRect();
    const top = windowY + toPhysicalPixels(domRect.top + border);
    const left = windowX + toPhysicalPixels(domRect.left + border);

    positions[hwnd] = {
      top,
      left,
      right: left + toPhysicalPixels(domRect.width - border * 2),
      bottom: top + toPhysicalPixels(domRect.height - border * 2),
    };
  });

  invoke(SeelenCommand.SetAppWindowsPositions, { positions });
}
