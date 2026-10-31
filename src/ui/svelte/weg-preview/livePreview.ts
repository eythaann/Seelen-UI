import type { PreviewRect } from "./hover.ts";

export interface LivePreviewFrame {
  targets: { hwnd: number; rect: PreviewRect; clip?: PreviewRect }[];
  opacity: number;
  anchor: PreviewRect | null;
}
export interface LivePreviewStatus {
  live: number[];
  visible: boolean;
  pointerInside: boolean;
}

/** The list's scrollable client area, excluding borders and scrollbars, under popup transforms. */
export function contentViewport(
  bounds: PreviewRect,
  box: {
    offsetWidth: number;
    offsetHeight: number;
    clientLeft: number;
    clientTop: number;
    clientWidth: number;
    clientHeight: number;
  },
): PreviewRect {
  const sx = box.offsetWidth > 0 ? (bounds.right - bounds.left) / box.offsetWidth : 0;
  const sy = box.offsetHeight > 0 ? (bounds.bottom - bounds.top) / box.offsetHeight : 0;
  const left = bounds.left + box.clientLeft * sx;
  const top = bounds.top + box.clientTop * sy;
  return { left, top, right: left + box.clientWidth * sx, bottom: top + box.clientHeight * sy };
}

/** Only visible cards consume native thumbnail slots; scrolling changes this selection. */
export function visiblePreviewTargets(
  targets: LivePreviewFrame["targets"],
  viewport: PreviewRect,
  preferred: number | null = null,
): LivePreviewFrame["targets"] {
  if (viewport.right <= viewport.left || viewport.bottom <= viewport.top) return [];
  const seen = new Set<number>();
  const visible = targets.flatMap((target) => {
    const { hwnd, rect } = target;
    const clip = {
      left: Math.max(rect.left, viewport.left, target.clip?.left ?? viewport.left),
      top: Math.max(rect.top, viewport.top, target.clip?.top ?? viewport.top),
      right: Math.min(rect.right, viewport.right, target.clip?.right ?? viewport.right),
      bottom: Math.min(rect.bottom, viewport.bottom, target.clip?.bottom ?? viewport.bottom),
    };
    if (
      seen.has(hwnd) || clip.right <= clip.left || clip.bottom <= clip.top
    ) return [];
    seen.add(hwnd);
    return [{ ...target, clip }];
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
