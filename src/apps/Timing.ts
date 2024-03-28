export function throttle<T extends anyFunction>(
  func: T,
  delay: number,
): T {
  let lastInvokeTime = 0;
  let timeoutId: ReturnType<typeof setTimeout> | null = null;

  return function (this: ThisParameterType<T>, ...args: Parameters<T>) {
    const now = Date.now();

    if (now - lastInvokeTime < delay) {
      if (timeoutId) {
        clearTimeout(timeoutId);
      }

      timeoutId = setTimeout(() => {
        lastInvokeTime = now;
        func.apply(this, args);
      }, delay);
    } else {
      lastInvokeTime = now;
      func.apply(this, args);
    }
  } as T;
}

export interface TimeoutIdRef {
  current: ReturnType<typeof setTimeout> | null;
}

export function debounce<T extends (...args: any[]) => any>(
  func: T,
  delay: number,
  timeoutId: TimeoutIdRef = { current: null },
): (...args: Parameters<T>) => void {
  return function debouncedFunction(this: ThisParameterType<T>, ...args: Parameters<T>) {
    const context = this;

    clearTimeout(timeoutId.current!);
    timeoutId.current = setTimeout(function () {
      func.apply(context, args);
    }, delay);
  };
}
