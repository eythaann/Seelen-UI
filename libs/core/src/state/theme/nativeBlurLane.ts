/** Bounded latest-value transport: a stranded old epoch cannot occupy the active epoch's only slot. */
export class NativeBlurLane<T> {
  private epoch = 0;
  private nextId = 0;
  private active = new Map<number, number>();
  private pending?: { epoch: number; value: T };

  constructor(
    private run: (value: T) => Promise<void>,
    private failed: (error: unknown) => void,
  ) {}

  advance(epoch: number): void {
    this.epoch = epoch;
    this.pending = undefined;
  }

  clear(): void {
    this.pending = undefined;
  }

  offer(epoch: number, value: T): void {
    if (epoch !== this.epoch) return;
    this.pending = { epoch, value };
    this.pump();
  }

  private pump(): void {
    const pending = this.pending;
    if (!pending || this.active.size >= 2 || [...this.active.values()].includes(pending.epoch)) return;
    this.pending = undefined;
    const id = ++this.nextId;
    this.active.set(id, pending.epoch);
    void Promise.resolve().then(() => this.run(pending.value)).catch((error) => {
      // A late rejection must not reset the current document/session or clear new visuals.
      if (pending.epoch === this.epoch) this.failed(error);
    }).finally(() => {
      this.active.delete(id); // Never clear another generation's active operation.
      this.pump();
    });
  }
}
