import type { Rect } from "@seelen-ui/types";
import { invoke, SeelenCommand, SeelenEvent, subscribe, type UnSubscriber } from "../../handlers/mod.ts";
import { debounce, type DebouncedFunction } from "../../utils/async.ts";

/** A hover popup hides after the cursor stays out of it (and its anchor) this long. */
const HOVER_OUT_GRACE_MS = 150;
/** Escape polling cadence: a hover popup never has keyboard focus, so it can't get key events. */
const ESCAPE_POLL_MS = 100;

/**
 * Dismissal of a popup opened by a `Hover` trigger: hides it once the cursor leaves the popup,
 * its anchor and the gap between them, or on Escape.
 *
 * Listeners only exist while armed (a hover-triggered popup is visible), so idle popups cost nothing.
 */
export class HoverDismissal {
  private anchor: Rect | null = null;
  private unlisten: Promise<UnSubscriber> | null = null;
  private escapePoll: ReturnType<typeof setInterval> | undefined;
  private polling = false;
  private readonly hideDelayed: DebouncedFunction<() => void>;

  constructor(
    private readonly popupRect: () => Rect,
    private readonly hide: () => Promise<void>,
  ) {
    this.hideDelayed = debounce(() => this.dismiss(), HOVER_OUT_GRACE_MS);
  }

  /** Arms (or re-anchors) the dismissal. */
  start(anchor: Rect | null): void {
    this.anchor = anchor;
    this.hideDelayed.cancel();
    if (this.unlisten) return;
    // Already broadcasted by the backend at ~10fps, in physical pixels.
    this.unlisten = subscribe(SeelenEvent.GlobalMouseMove, ({ payload: [x, y] }) => {
      if (isInsideHoverBridge(x, y, this.popupRect(), this.anchor)) {
        this.hideDelayed.cancel();
      } else {
        this.hideDelayed();
      }
    });
    this.escapePoll = setInterval(() => this.pollEscape(), ESCAPE_POLL_MS);
  }

  /** Releases every listener. */
  stop(): void {
    this.hideDelayed.cancel();
    clearInterval(this.escapePoll);
    this.escapePoll = undefined;
    const unlisten = this.unlisten;
    this.unlisten = null;
    unlisten?.then((unsubscribe) => unsubscribe()).catch(console.error);
  }

  private dismiss(): void {
    this.stop();
    this.hide().catch(console.error);
  }

  private async pollEscape(): Promise<void> {
    if (this.polling) return;
    this.polling = true;
    try {
      if (this.escapePoll !== undefined && await invoke(SeelenCommand.GetKeyState, { key: "Escape" })) {
        this.dismiss();
      }
    } catch (error) {
      console.error("Failed to read the Escape key state", error);
    } finally {
      this.polling = false;
    }
  }
}

/** Inside the bounding box of the popup and its anchor, so the gap between them is crossable. */
export function isInsideHoverBridge(x: number, y: number, popup: Rect, anchor: Rect | null): boolean {
  const left = anchor ? Math.min(popup.left, anchor.left) : popup.left;
  const top = anchor ? Math.min(popup.top, anchor.top) : popup.top;
  const right = anchor ? Math.max(popup.right, anchor.right) : popup.right;
  const bottom = anchor ? Math.max(popup.bottom, anchor.bottom) : popup.bottom;
  return x >= left && x <= right && y >= top && y <= bottom;
}
