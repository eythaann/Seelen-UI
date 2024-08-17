export class CallbacksManager {
  callbacks: Record<string, () => void> = {};

  add(cb: () => void, key: string) {
    this.callbacks[key] = cb;
  }

  remove(key: string) {
    delete this.callbacks[key];
  }

  execute() {
    Object.values(this.callbacks).forEach((cb) => cb());
  }
}