type EffectResult = void | (() => void) | Promise<void>;

/**
 * Runs `fn` immediately and re-runs it whenever the reactive values it reads change.
 * Meant for module scope (outside components). Returns a
 * promise that resolves once the effect has run at least once. If the effect callback
 * is async, the promise waits until that first run settles.
 *
 * Replaces the common pattern:
 * ```typescript
 * await doSomething(value);
 * $effect.root(() => {
 *   $effect(() => {
 *     doSomething(value);
 *   });
 * });
 * ```
 *
 * Notes:
 * - Only the synchronous part of `fn` (before the first `await`) is tracked by Svelte.
 * - Errors on the first run reject the promise; later runs log to the console.
 * - Sync callbacks can return a cleanup function, like a regular `$effect`.
 *
 * @example
 * ```typescript
 * await rootEffectAsync(() => Widget.self.setPosition(desktopRect));
 * ```
 */
export function rootEffectAsync(fn: () => EffectResult): Promise<void> {
  return new Promise<void>((resolve, reject) => {
    let firstRun = true;

    $effect.root(() => {
      $effect(() => {
        const isFirst = firstRun;
        firstRun = false;

        let result: EffectResult;
        try {
          result = fn();
        } catch (error) {
          if (!isFirst) throw error;
          reject(error);
          return;
        }

        if (result instanceof Promise) {
          result.then(
            () => isFirst && resolve(),
            (error) => (isFirst ? reject(error) : console.error(error)),
          );
          return;
        }

        if (isFirst) resolve();
        return result;
      });
    });
  });
}
