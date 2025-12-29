export class PersistentRune<T> {
  private _value: T;
  private storeKey: string;

  constructor(storeKey: string, initial: T) {
    this.storeKey = storeKey;

    // Try to load from localStorage, fallback to initial value
    const stored = this.loadFromStorage();
    this._value = $state(stored ?? initial);
  }

  private loadFromStorage(): T | null {
    try {
      const item = localStorage.getItem(this.storeKey);
      if (item === null) {
        return null;
      }
      return JSON.parse(item) as T;
    } catch (error) {
      console.error(`Failed to load from localStorage (key: ${this.storeKey}):`, error);
      return null;
    }
  }

  private saveToStorage(value: T): void {
    try {
      localStorage.setItem(this.storeKey, JSON.stringify(value));
    } catch (error) {
      console.error(`Failed to save to localStorage (key: ${this.storeKey}):`, error);
    }
  }

  get value(): T {
    return this._value;
  }

  set value(value: T) {
    this._value = value;
    this.saveToStorage(value);
  }
}

/**
 * Helper function to create a PersistentRune
 *
 * @example
 * const count = persistentRune('counter', 0);
 * count.value++; // Automatically saved to localStorage
 */
export function persistentRune<T>(storeKey: string, initial: T): PersistentRune<T> {
  return new PersistentRune(storeKey, initial);
}
