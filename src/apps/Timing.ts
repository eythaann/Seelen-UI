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