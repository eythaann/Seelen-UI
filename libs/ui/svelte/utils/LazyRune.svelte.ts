/**
 * Structure done to work with async event programming
 *
 * The LazyRune double-check pattern ensures that if an event updates
 * the value during initialization, that event value won't be overwritten by
 * the stale initial fetch.
 *
 * How to use:
 *
 * 1. Create lazy rune with async initializer
 * 2. Set up event listeners that can fire anytime
 * 3. Initialize - won't overwrite if event already fired during fetch
 *
 * @example
 * ```typescript
 * // 1. Create lazy rune with async initializer
 * const colors = lazyRune(async () => (await UIColors.getAsync()).inner);
 *
 * // 2. Set up event listeners that can fire anytime
 * await UIColors.onChange(colors.setByPayload);
 *
 * // 3. Initialize - won't overwrite if event already fired
 * await colors.init();
 *
 * // 4. Use the value (will be reactive in Svelte components)
 * $effect(() => {
 *   console.log(colors.value);
 * });
 * ```
 */
export class LazyRune<T> {
  private _value = $state<T>();
  private initialized = false;

  constructor(private initializer: () => Promise<T> | T) {
    this.setByPayload = this.setByPayload.bind(this);
  }

  /**
   * Gets the current value. Throws error if not initialized yet.
   * This property is reactive and will trigger updates in Svelte components.
   */
  get value(): T {
    if (!this.initialized) {
      throw new Error("LazyRune was not initialized");
    }
    return this._value as T;
  }

  /**
   * Sets the value and marks as initialized.
   * This will trigger reactivity in Svelte components.
   */
  set value(value: T) {
    this.initialized = true;
    this._value = value;
  }

  /**
   * Will call the initializer and set the value if not already set
   * via another setters.
   *
   * This uses a double-check pattern to prevent race conditions:
   * - Check if initialized
   * - Call initializer
   * - Check again after await (event may have set value during fetch)
   * - Only set if still not initialized
   */
  public async init(): Promise<void> {
    if (!this.initialized) {
      const awaited = await this.initializer();
      // double check - event may have fired during async initialization
      if (!this.initialized) {
        this.value = awaited;
      }
    }
  }

  /**
   * Utility function to be used as event handler.
   * Useful for Tauri event listeners.
   *
   * @example
   * ```typescript
   * await UIColors.onChange(colors.setByPayload);
   * ```
   */
  public setByPayload({ payload }: { payload: T }): void {
    this.value = payload;
  }

  /**
   * Checks if the rune has been initialized.
   * Useful for conditional rendering or logic.
   */
  public isInitialized(): boolean {
    return this.initialized;
  }
}

/**
 * Factory function to create a new LazyRune instance.
 *
 * @param initial - Function that returns the initial value (can be async)
 * @returns A new LazyRune instance
 *
 * @example
 * ```typescript
 * const systemColors = lazyRune(async () => {
 *   const colors = await UIColors.getAsync();
 *   return colors.inner;
 * });
 *
 * // Set up event listeners
 * await UIColors.onChange(systemColors.setByPayload);
 *
 * // Initialize
 * await systemColors.init();
 * ```
 */
export function lazyRune<T>(initial: () => Promise<T> | T): LazyRune<T> {
  return new LazyRune(initial);
}
