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
interface LivePreviewBridge {
  update(session: string, frame: LivePreviewFrame): Promise<LivePreviewStatus>;
  clear(session: string): Promise<unknown>;
}

/** Layout follows animation frames; the video itself stays in DWM, with no screenshot traffic. */
export class LivePreviewController {
  private token: string | null = null;
  private pending = false;
  private latest: { frame: LivePreviewFrame; now: number } | null = null;
  private lastKey = "";
  private lastSent = -Infinity;

  constructor(
    private readonly bridge: LivePreviewBridge,
    private readonly onStatus: (status: LivePreviewStatus) => void,
    private readonly newToken: () => string = () => crypto.randomUUID(),
  ) {}

  update(frame: LivePreviewFrame, now: number): void {
    this.latest = { frame, now };
    if (this.pending) return;
    const key = JSON.stringify(frame);
    // Stable geometry only needs a light heartbeat / pointer check, not per-video-frame IPC.
    if (key === this.lastKey && now - this.lastSent < 80) return;
    this.latest = null;
    const token = this.token ??= this.newToken();
    this.lastKey = key;
    this.lastSent = now;
    this.pending = true;
    void this.bridge.update(token, frame).then((status) => {
      if (this.token === token) this.onStatus(status);
    }).catch((error: unknown) => {
      if (this.token === token) {
        this.lastKey = "";
        console.error("Live preview update failed", error);
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
    if (token) void this.bridge.clear(token).catch(console.error);
  }
}
