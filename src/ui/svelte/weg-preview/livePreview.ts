import type { PreviewRect } from "./hover.ts";

export interface LivePreviewFrame {
  targets: { hwnd: number; rect: PreviewRect }[];
  opacity: number;
  anchor: PreviewRect | null;
}
export interface LivePreviewStatus {
  live: number[];
  visible: boolean;
  pointerInside: boolean;
}

/** Only visible cards consume native thumbnail slots; scrolling changes this selection. */
export function visiblePreviewTargets(
  targets: LivePreviewFrame["targets"],
  viewport: PreviewRect,
  preferred: number | null = null,
): LivePreviewFrame["targets"] {
  if (viewport.right <= viewport.left || viewport.bottom <= viewport.top) return [];
  const seen = new Set<number>();
  const visible = targets.filter(({ hwnd, rect }) => {
    if (
      seen.has(hwnd) || rect.right <= rect.left || rect.bottom <= rect.top ||
      rect.right <= viewport.left || rect.left >= viewport.right ||
      rect.bottom <= viewport.top || rect.top >= viewport.bottom
    ) return false;
    seen.add(hwnd);
    return true;
  });
  if (preferred !== null) visible.sort((a, b) => Number(b.hwnd === preferred) - Number(a.hwnd === preferred));
  return visible.slice(0, 16);
}

interface LivePreviewBridge {
  update(session: string, frame: LivePreviewFrame): Promise<LivePreviewStatus>;
  clear(session: string): Promise<unknown>;
  recover?(session: string, frame: LivePreviewFrame): Promise<LivePreviewStatus>;
}

/** Layout follows animation frames; the video itself stays in DWM, with no screenshot traffic. */
export class LivePreviewController {
  private token: string | null = null;
  private pending = false;
  private latest: { frame: LivePreviewFrame; now: number } | null = null;
  private lastKey = "";
  private lastSent = -Infinity;
  private retryAfter = -Infinity;

  constructor(
    private readonly bridge: LivePreviewBridge,
    private readonly onStatus: (status: LivePreviewStatus, frame: LivePreviewFrame) => void,
    private readonly newToken: () => string = () => crypto.randomUUID(),
  ) {}

  update(frame: LivePreviewFrame, now: number): void {
    this.latest = { frame, now };
    if (this.pending || now < this.retryAfter) return;
    const key = JSON.stringify(frame);
    // Stable geometry only needs a light heartbeat / pointer check, not per-video-frame IPC.
    if (key === this.lastKey && now - this.lastSent < 80) return;
    this.latest = null;
    const token = this.token ??= this.newToken();
    this.lastKey = key;
    this.lastSent = now;
    this.pending = true;
    void this.bridge.update(token, frame).then((status) => {
      if (this.token === token) this.onStatus(status, frame);
    }).catch(async (error: unknown) => {
      if (this.token === token) {
        this.lastKey = "";
        this.retryAfter = this.lastSent + 250;
        console.error("Live preview update failed", error);
        // A rejected target must not strand the popup without pointer/lifetime state.
        if (this.bridge.recover) {
          try {
            const status = await this.bridge.recover(token, frame);
            if (this.token === token) this.onStatus(status, frame);
          } catch (recoveryError) {
            console.error("Live preview recovery failed", recoveryError);
          }
        }
      }
    }).finally(() => {
      this.pending = false;
      const next = this.latest;
      this.latest = null;
      if (next) this.update(next.frame, next.now);
    });
  }

  clear(): void {
    const token = this.token;
    this.token = null;
    this.latest = null;
    this.lastKey = "";
    this.lastSent = -Infinity;
    this.retryAfter = -Infinity;
    if (token) void this.bridge.clear(token).catch(console.error);
  }
}
