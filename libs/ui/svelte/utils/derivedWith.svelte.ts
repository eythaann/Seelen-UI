import { deepEqual } from "../../deepEqual";

const UNSET = Symbol("unset");

export function derivedWith<T>(
  compute: () => T,
  equals: (a: T, b: T) => boolean = deepEqual,
): { readonly value: T } {
  let previous: T | typeof UNSET = UNSET;
  const value = $derived.by(() => {
    const next = compute();
    if (previous !== UNSET && equals(previous, next)) return previous;
    previous = next;
    return next;
  });
  return {
    get value() {
      return value;
    },
  };
}
