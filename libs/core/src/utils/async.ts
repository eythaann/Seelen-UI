// deno-lint-ignore no-explicit-any
export interface DebouncedFunction<T extends (...args: any[]) => any> {
  (...args: Parameters<T>): void;
  cancel(): void;
  flush(): void;
  pending(): boolean;
}

// deno-lint-ignore no-explicit-any
export function debounce<T extends (...args: any[]) => any>(
  fn: T,
  delay: number,
): DebouncedFunction<T> {
  let timeout: ReturnType<typeof setTimeout> | null = null;
  let lastArgs: Parameters<T> | null = null;

  const debounced = (...args: Parameters<T>) => {
    lastArgs = args;

    if (timeout) {
      clearTimeout(timeout);
    }

    timeout = setTimeout(() => {
      if (lastArgs) {
        fn(...lastArgs);
        lastArgs = null;
      }
      timeout = null;
    }, delay);
  };

  debounced.cancel = () => {
    if (timeout) {
      clearTimeout(timeout);
      timeout = null;
    }
    lastArgs = null;
  };

  debounced.flush = () => {
    if (timeout) {
      clearTimeout(timeout);
      timeout = null;
    }

    if (lastArgs) {
      fn(...lastArgs);
      lastArgs = null;
    }
  };

  debounced.pending = () => {
    return timeout !== null;
  };

  return debounced as DebouncedFunction<T>;
}

export class Mutex<T> {
  constructor(private _rawValue: T) {}

  private lock: {
    __promise: Promise<() => void>;
    __resolve: () => void;
    __reject: () => void;
  } | null = null;

  async acquire(): Promise<Guard<T>> {
    if (this.lock) {
      await this.lock.__promise;
    }

    // deno-lint-ignore no-explicit-any
    let __resolve: any;
    // deno-lint-ignore no-explicit-any
    let __reject: any;
    const __promise = new Promise<() => void>((resolve, reject) => {
      __resolve = resolve;
      __reject = reject;
    });

    this.lock = {
      __promise,
      __resolve,
      __reject,
    };

    return new Guard(this._rawValue, () => {
      this.lock = null;
      __resolve();
    });
  }

  async runExclusive<U>(fn: (v: T) => U): Promise<U> {
    const guard = await this.acquire();
    try {
      return await fn(guard.value);
    } finally {
      guard.release();
    }
  }
}

export class Guard<T> {
  constructor(
    public value: T,
    public release: () => void,
  ) {}
}
