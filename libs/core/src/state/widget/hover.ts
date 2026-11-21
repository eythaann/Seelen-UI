import type { Rect } from "@seelen-ui/types";
import { debounce, type DebouncedFunction } from "../../utils/async.ts";

/** A hover popup hides after the cursor stays out of it (and its anchor) this long. */
const HOVER_OUT_GRACE_MS = 150;

interface HoverPointer {
  subscribe(onMove: (x: number, y: number) => void): Promise<() => void>;
  position(): Promise<{ x: number; y: number }>;
}

/**
 * Dismissal of a popup opened by a `Hover` trigger: hides it once the cursor leaves the popup,
 * its anchor and the gap between them.
 *
 * The listener only exists while armed (a hover-triggered popup is visible), so idle popups cost nothing.
 */
export class HoverDismissal {
  private anchor: Rect | null = null;
  private unlisten: Promise<() => void> | null = null;
  private session: object | null = null;
  private readonly hideDelayed: DebouncedFunction<() => void>;

  constructor(
    private readonly popupRect: () => Rect,
    private readonly hide: () => Promise<void>,
    private readonly pointer: HoverPointer,
  ) {
    this.hideDelayed = debounce(() => {
      this.stop();
      this.hide().catch(console.error);
    }, HOVER_OUT_GRACE_MS);
  }

  /** Arms (or re-anchors) the dismissal. */
  start(anchor: Rect | null): void {
    this.stop();
    this.anchor = anchor;
    const session = this.session = {};
    let moves = 0;
    // Already broadcasted by the backend at ~10fps, in physical pixels.
    this.unlisten = this.pointer.subscribe((x, y) => {
      if (this.session !== session) return;
      moves++;
      this.checkPosition(x, y);
    });
    // Listen first, then sample: the cursor may already be stationary outside a slow-opening popup.
    this.unlisten.then(async () => {
      if (this.session !== session) return;
      const revision = moves;
      const { x, y } = await this.pointer.position();
      // A later move or another trigger takes precedence over this asynchronous sample.
      if (this.session === session && moves === revision) this.checkPosition(x, y);
    }).catch(console.error);
  }

  private checkPosition(x: number, y: number): void {
    if (isInsideHoverBridge(x, y, this.popupRect(), this.anchor)) {
      this.hideDelayed.cancel();
    } else if (!this.hideDelayed.pending()) {
      this.hideDelayed();
    }
  }

  /** Releases the listener. */
  stop(): void {
    this.session = null;
    this.hideDelayed.cancel();
    const unlisten = this.unlisten;
    this.unlisten = null;
    unlisten?.then((unsubscribe) => unsubscribe()).catch(console.error);
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
