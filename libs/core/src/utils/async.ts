// deno-lint-ignore no-explicit-any
export function debounce<T extends (...args: any[]) => any>(
  fn: T,
  delay: number,
): T {
  let timeout: ReturnType<typeof setTimeout> | null = null;
  const debounced = (...args: Parameters<T>) => {
    if (timeout) {
      clearTimeout(timeout);
    }
    timeout = setTimeout(() => {
      fn(...args);
    }, delay);
  };
  return debounced as T;
}
