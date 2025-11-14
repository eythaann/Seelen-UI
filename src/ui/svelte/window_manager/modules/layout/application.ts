import { invoke, SeelenCommand } from "@seelen-ui/lib";
import type { Rect } from "@seelen-ui/lib/types";
import { toPhysicalPixels } from "@shared";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { State } from "../shared/state.svelte";

export async function requestPositioningOfLeaves(state: State) {
  const { x: windowX, y: windowY } = await getCurrentWindow().outerPosition();

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

  await invoke(SeelenCommand.SetAppWindowsPositions, { positions });
}
