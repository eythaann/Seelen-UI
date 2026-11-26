import { invoke, SeelenCommand, SeelenEvent, subscribe, type UnSubscriber } from "@seelen-ui/lib";
import type { UserAppWindowPreview, UserAppWindowPreviewDelta } from "@seelen-ui/lib/types";

type WindowPreviews = Record<string, UserAppWindowPreview>;

interface PreviewSession {
  pending: Map<string, UserAppWindowPreviewDelta> | null;
  unsubscribe?: UnSubscriber;
}

export class PreviewSnapshots {
  private snapshots = $state<WindowPreviews>({});
  private session: PreviewSession | null = null;

  get value(): WindowPreviews {
    return this.snapshots;
  }

  start(targets?: ReadonlySet<string>): void {
    if (this.session) return;
    const session: PreviewSession = { pending: new Map() };
    this.session = session;
    this.load(session, targets).catch(console.error);
  }

  stop(): void {
    const session = this.session;
    this.session = null;
    session?.unsubscribe?.();
  }

  private async load(session: PreviewSession, targets?: ReadonlySet<string>): Promise<void> {
    const accepts = (hwnd: string) => !targets || targets.has(hwnd);
    try {
      const unsubscribe = await subscribe(SeelenEvent.UserAppWindowPreviewDelta, ({ payload }) => {
        if (this.session !== session || !accepts(payload.hwnd)) return;
        if (session.pending) {
          session.pending.set(payload.hwnd, payload);
        } else {
          applyDelta(this.snapshots, payload);
        }
      });
      if (this.session !== session) {
        unsubscribe();
        return;
      }
      session.unsubscribe = unsubscribe;
    } catch (error) {
      if (this.session === session) this.stop();
      throw error;
    }

    try {
      const snapshot = await invoke(SeelenCommand.GetUserAppWindowsPreviews);
      if (this.session !== session) return;
      const visible = targets
        ? Object.fromEntries(Object.entries(snapshot).filter(([hwnd]) => accepts(hwnd)))
        : snapshot;
      // Updates received while the initial snapshot was in flight take precedence.
      session.pending?.forEach((delta) => applyDelta(visible, delta));
      this.snapshots = visible;
    } catch (error) {
      if (this.session === session) session.pending?.forEach((delta) => applyDelta(this.snapshots, delta));
      throw error;
    } finally {
      session.pending = null;
    }
  }
}

function applyDelta(snapshots: WindowPreviews, delta: UserAppWindowPreviewDelta): void {
  const { hwnd } = delta;
  if (delta.type === "Removed") {
    delete snapshots[hwnd];
  } else if (snapshots[hwnd]?.hash !== delta.preview.hash) {
    snapshots[hwnd] = delta.preview;
  }
}
