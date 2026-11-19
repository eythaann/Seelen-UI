/** Serialize clicks for one source, and ignore replies belonging to an earlier source. */
export class MediaCommand {
  private target = "";
  private generation = 0;
  private pending = false;

  constructor(private onState: (state: { pending: boolean; failed: boolean }) => void, private timeoutMs = 8000) {}

  setTarget(target: string): void {
    if (target === this.target) return;
    this.target = target;
    this.generation++;
    this.pending = false;
    this.onState({ pending: false, failed: false });
  }

  async run(command: () => Promise<unknown>): Promise<boolean> {
    if (this.pending) return false;
    const generation = this.generation;
    this.pending = true;
    this.onState({ pending: true, failed: false });
    let timeout: ReturnType<typeof setTimeout> | undefined;
    try {
      await Promise.race([
        command(),
        new Promise((_, reject) => {
          timeout = setTimeout(() => reject(new Error("Media command timed out")), this.timeoutMs);
        }),
      ]);
      if (generation === this.generation) this.onState({ pending: false, failed: false });
      return true;
    } catch {
      if (generation === this.generation) this.onState({ pending: false, failed: true });
      return false;
    } finally {
      clearTimeout(timeout);
      if (generation === this.generation) this.pending = false;
    }
  }
}
