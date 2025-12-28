interface Transition {
  readonly loading: boolean;
  readonly error: unknown;
  start: (cb: () => Promise<void>) => Promise<void>;
  clearError: () => void;
}

export function useTransition(): Transition {
  let loading = $state(false);
  let error = $state();

  async function startTransition(cb: () => Promise<void>): Promise<void> {
    loading = true;
    error = undefined;

    try {
      await cb();
    } catch (e: unknown) {
      console.error(e);
      error = e;
    } finally {
      loading = false;
    }
  }

  function clearError() {
    error = null;
  }

  return {
    get loading() {
      return loading;
    },
    get error() {
      return error;
    },
    start: startTransition,
    clearError,
  };
}
