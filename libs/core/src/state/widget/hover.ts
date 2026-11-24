import type { Rect } from "@seelen-ui/types";
import { invoke, SeelenCommand, SeelenEvent, subscribe, type UnSubscriber } from "../../handlers/mod.ts";
import { debounce, type DebouncedFunction } from "../../utils/async.ts";

/** A hover popup hides after the cursor stays out of it (and its anchor) this long. */
const HOVER_OUT_GRACE_MS = 150;

/**
 * Dismissal of a popup opened by a `Hover` trigger: hides it once the cursor leaves the popup,
 * its anchor and the gap between them.
 *
 * The listener only exists while armed (a hover-triggered popup is visible), so idle popups cost nothing.
 */
export class HoverDismissal {
  private anchor: Rect | null = null;
  private unlisten: Promise<UnSubscriber> | null = null;
  private pointerRevision = 0;
  private readonly hideDelayed: DebouncedFunction<() => void>;

  constructor(
    private readonly popupRect: () => Rect,
    private readonly hide: () => Promise<void>,
    private readonly mouse = {
      position: () => invoke(SeelenCommand.GetMousePosition),
      listen: (onMove: (x: number, y: number) => void) =>
        subscribe(SeelenEvent.GlobalMouseMove, ({ payload: [x, y] }) => onMove(x, y)),
    },
  ) {
    this.hideDelayed = debounce(() => {
      this.stop();
      this.hide().catch(console.error);
    }, HOVER_OUT_GRACE_MS);
  }

  /** Arms (or re-anchors) the dismissal. */
  start(anchor: Rect | null): void {
    this.anchor = anchor;
    this.hideDelayed.cancel();
    if (!this.unlisten) {
      const listener = this.mouse.listen((x, y) => {
        if (this.unlisten === listener) this.checkPointer(x, y);
      });
      this.unlisten = listener;
    }

    // Movement events only report changes. The pointer can already be outside when a slow popup opens.
    const revision = ++this.pointerRevision;
    this.unlisten.then(() => this.mouse.position()).then(([x, y]) => {
      if (this.pointerRevision === revision) this.checkPointer(x, y);
    }).catch(console.error);
  }

  /** Releases the listener. */
  stop(): void {
    this.pointerRevision++;
    this.hideDelayed.cancel();
    const unlisten = this.unlisten;
    this.unlisten = null;
    unlisten?.then((unsubscribe) => unsubscribe()).catch(console.error);
  }

  private checkPointer(x: number, y: number): void {
    this.pointerRevision++;
    if (isInsideHoverBridge(x, y, this.popupRect(), this.anchor)) {
      this.hideDelayed.cancel();
    } else if (!this.hideDelayed.pending()) {
      this.hideDelayed();
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
