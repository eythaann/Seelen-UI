const hasOwn = Object.prototype.hasOwnProperty;
export function deepEqual(a: unknown, b: unknown): boolean {
  if (a === b) return true;

  if (a && b && typeof a === "object" && typeof b === "object") {
    const aIsArr = Array.isArray(a);
    if (aIsArr !== Array.isArray(b)) return false;

    if (aIsArr) {
      const arrA = a as unknown[],
        arrB = b as unknown[];
      let i = arrA.length;
      if (i !== arrB.length) return false;
      while (i-- !== 0) if (!deepEqual(arrA[i], arrB[i])) return false;
      return true;
    }

    const objA = a as Record<string, unknown>,
      objB = b as Record<string, unknown>;
    const keys = Object.keys(objA);
    let i = keys.length;
    if (i !== Object.keys(objB).length) return false;

    while (i-- !== 0) {
      const k = keys[i]!;
      if (!hasOwn.call(objB, k) || !deepEqual(objA[k], objB[k])) return false;
    }
    return true;
  }

  // NaN === NaN
  return a !== a && b !== b;
}
