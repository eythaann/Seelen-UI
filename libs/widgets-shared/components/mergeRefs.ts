import { Component, Ref, RefCallback } from 'preact';

export function assignRef<T>(
  ref: Ref<T> | undefined | null,
  value: T | null,
): ReturnType<RefCallback<T>> {
  if (typeof ref === 'function') {
    return ref(value);
  } else if (ref) {
    ref.current = value;
  }
}

export function mergeRefs<T>(refs: (Ref<T> | undefined)[]): Ref<T> {
  return (value: T | null) => {
    const cleanups: ((...args: any[]) => void)[] = [];

    for (const ref of refs) {
      const cleanup = assignRef(ref, (value as Component)?.base as T || value);
      const isCleanup = typeof cleanup === 'function';
      cleanups.push(isCleanup ? cleanup : () => assignRef(ref, null));
    }

    return (...args: any[]) => {
      for (const cleanup of cleanups) {
        cleanup(...args);
      }
    };
  };
}
