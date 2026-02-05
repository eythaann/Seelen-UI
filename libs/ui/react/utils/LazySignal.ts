import { Signal } from "@preact/signals";

/**
 * Structure done to work with async event programming
 */
export class LazySignal<T> extends Signal<T> {
  private initialized = false;
  private intializing: Promise<Signal<T>> | null = null;

  constructor(private initializer: () => Promise<T>) {
    super();
    this.setByPayload = this.setByPayload.bind(this);
  }

  get value(): T {
    if (!this.initialized) {
      throw new Error("LazySignal was not initialized");
    }
    return super.value;
  }

  set value(value: T) {
    this.initialized = true;
    super.value = value;
  }

  /**
   * Will call the initializer and set the value if not already set
   * via another setters.
   */
  public init(): Promise<Signal<T>> {
    if (!this.intializing) {
      this.intializing = this.initializer().then((initial) => {
        if (!this.initialized) {
          this.value = initial;
        }
        return this;
      });
    }
    return this.intializing;
  }

  /**
   * Utility function to be used as event handler
   */
  public setByPayload({ payload }: { payload: T }) {
    this.value = payload;
  }
}

/**
 * The LazySignal double-check pattern ensures that if an event updates
 * the value during initialization, that event value won't be overwritten by
 * the stale initial fetch.
 *
 * How to use:
 *
 * 1. Create lazy signal with async initializer
 * 2. Set up event listeners that can fire anytime
 * 3. Initialize - won't overwrite if event already fired during fetch
 */
export function lazySignal<T>(initial: () => Promise<T>): LazySignal<T> {
  return new LazySignal(initial);
}
