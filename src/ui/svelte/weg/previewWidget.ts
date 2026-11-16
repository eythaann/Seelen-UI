import { invoke, SeelenCommand, Widget } from "@seelen-ui/lib";
import { Alignment, SeelenWegSide, TriggerSource, type UserAppWindow, type WidgetId } from "@seelen-ui/lib/types";
import { settingsState } from "./state/settings.svelte.ts";

/** Hover time on a dock item before its preview opens. */
const HOVER_DWELL_MS = 350;
/**
 * Moving from the item whose preview is open straight onto another item (within this time)
 * switches the preview instantly, without the dwell, like the native taskbar.
 */
const HOVER_SWITCH_MS = 300;

let hoverPreviewItem: HTMLElement | null = null;
let hoverPreviewLeftAt = -Infinity;

/** Delay before a hovered item opens its preview. */
export function hoverPreviewDelay(): number {
  return performance.now() - hoverPreviewLeftAt < HOVER_SWITCH_MS ? 0 : HOVER_DWELL_MS;
}

/** Call when the cursor leaves a dock item. */
export function onHoverPreviewItemLeave(itemEl: HTMLElement): void {
  if (itemEl === hoverPreviewItem) {
    hoverPreviewLeftAt = performance.now();
  }
}

export async function triggerPreviewWidget(
  itemEl: HTMLElement,
  windows: UserAppWindow[],
  hover = false,
): Promise<void> {
  hoverPreviewItem = hover ? itemEl : null;
  hoverPreviewLeftAt = -Infinity;
  const dockSide = settingsState.position;

  const origin = await Widget.self.window.outerPosition();
  if (!itemEl.isConnected || (hover && !itemEl.matches(":hover"))) return;
  const elRect = itemEl.getBoundingClientRect();

  const toPhysical = (n: number) => Math.round(n * globalThis.devicePixelRatio);
  const viewRect = {
    left: origin.x + toPhysical(elRect.left),
    top: origin.y + toPhysical(elRect.top),
    right: origin.x + toPhysical(elRect.right),
    bottom: origin.y + toPhysical(elRect.bottom),
  };

  let x: number;
  let y: number;
  let alignX: Alignment;
  let alignY: Alignment;

  switch (dockSide) {
    case SeelenWegSide.Bottom:
      x = Math.round((viewRect.left + viewRect.right) / 2);
      y = viewRect.top;
      alignX = Alignment.Center;
      alignY = Alignment.End;
      break;
    case SeelenWegSide.Top:
      x = Math.round((viewRect.left + viewRect.right) / 2);
      y = viewRect.bottom;
      alignX = Alignment.Center;
      alignY = Alignment.Start;
      break;
    case SeelenWegSide.Left:
      x = viewRect.right;
      y = Math.round((viewRect.top + viewRect.bottom) / 2);
      alignX = Alignment.Start;
      alignY = Alignment.Center;
      break;
    case SeelenWegSide.Right:
      x = viewRect.left;
      y = Math.round((viewRect.top + viewRect.bottom) / 2);
      alignX = Alignment.End;
      alignY = Alignment.Center;
      break;
  }

  return invoke(SeelenCommand.TriggerWidget, {
    payload: {
      id: "@seelen/weg-preview" as WidgetId,
      desiredPosition: { x, y },
      alignX,
      alignY,
      source: hover ? TriggerSource.Hover : TriggerSource.Click,
      anchorRect: viewRect,
      customArgs: {
        hwnds: windows.map((w) => w.hwnd),
      },
    },
  });
}
