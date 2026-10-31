import type { PreviewRect } from "./hover.ts";

export interface PeekStatus {
  active: boolean;
  committed: boolean;
  hwnd: number | null;
}

export interface PeekBridge {
  begin(hwnd: number, session: string, rect: PreviewRect): Promise<PeekStatus>;
  poll(session: string, rect: PreviewRect | null): Promise<PeekStatus>;
  cancel(session: string): Promise<unknown>;
  commit(session: string): Promise<boolean>;
  activate(hwnd: number): Promise<unknown>;
}

interface Session {
  hwnd: number;
  token: string;
  ready: Promise<PeekStatus>;
}

/** The native lease survives pointer transfer from the thumbnail to the real window. */
export class WindowPeekController {
  private candidate: number | null = null;
  private since = 0;
  private blocked: number | null = null;
  private session: Session | null = null;
  private busy = false;
  private committing = false;

  constructor(
    private readonly bridge: PeekBridge,
    private readonly onCommitted: () => void,
    private readonly token: () => string = () => crypto.randomUUID(),
  ) {}

  get holding(): boolean {
    return this.session !== null || this.committing;
  }

  hover(hwnd: number | null, now: number): void {
    if (this.candidate === hwnd) return;
    this.candidate = hwnd;
    this.since = now;
    this.blocked = null;
    // Leaving a thumbnail alone is intentional: the native hit test follows the real window.
    if (hwnd !== null && this.session?.hwnd !== hwnd && !this.committing) this.cancelSession();
  }

  async tick(rect: PreviewRect | null, now: number): Promise<void> {
    if (this.busy || this.committing) return;
    this.busy = true;
    try {
      if (
        !this.session && this.candidate !== null && this.blocked !== this.candidate && rect && now - this.since >= 450
      ) {
        const hwnd = this.candidate;
        const token = this.token();
        const ready = this.bridge.begin(hwnd, token, rect);
        const session = { hwnd, token, ready };
        this.session = session;
        try {
          const status = await ready;
          if (this.session === session) this.accept(session, status);
        } catch (error) {
          if (this.session === session) {
            this.cancelSession();
            this.blocked = hwnd;
          }
          throw error;
        }
      } else if (this.session) {
        const session = this.session;
        const status = await this.bridge.poll(session.token, this.candidate === session.hwnd ? rect : null);
        if (this.session === session && !this.committing) this.accept(session, status);
      }
    } finally {
      this.busy = false;
    }
  }

  private accept(session: Session, status: PeekStatus): void {
    if (status.active) return;
    this.session = null;
    this.blocked = session.hwnd;
    if (status.committed) this.onCommitted();
  }

  async commit(hwnd: number): Promise<boolean> {
    if (this.committing) return false;
    this.committing = true;
    try {
      const session = this.session;
      if (session?.hwnd === hwnd) {
        await session.ready;
        // Focus the exact HWND already on screen; never toggle/minimize it during handoff.
        const committed = await this.bridge.commit(session.token);
        if (committed) {
          this.session = null;
          this.onCommitted();
        }
        return committed;
      }
      this.cancelSession();
      await this.bridge.activate(hwnd);
      this.onCommitted();
      return true;
    } finally {
      this.committing = false;
    }
  }

  cancel(): void {
    this.candidate = null;
    this.blocked = null;
    this.cancelSession();
  }

  private cancelSession(): void {
    const session = this.session;
    this.session = null;
    if (session) void this.bridge.cancel(session.token).catch(console.error);
  }
}
