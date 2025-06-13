import { getCurrentWindow } from '@tauri-apps/api/window';
import {
  debounce,
  DebouncedFunc,
  DebouncedFuncLeading,
  isEqual,
  throttle,
  ThrottleSettings,
} from 'lodash';
import { useEffect, useMemo, useRef } from 'react';

export function useWindowFocusChange(cb: (focused: boolean) => void) {
  useEffect(() => {
    const promise = getCurrentWindow().onFocusChanged((event) => {
      cb(event.payload);
    });
    return () => {
      promise.then((unlisten) => unlisten());
    };
  }, []);
}

/** Will reset the interval on deps change */
export function useInterval(cb: () => void, ms: number, deps: any[] = []) {
  const ref = useRef<number | null>(null);
  const clearLastInterval = () => {
    if (ref.current) {
      clearInterval(ref.current);
    }
  };
  useEffect(() => {
    clearLastInterval();
    ref.current = window.setInterval(cb, ms);
    return clearLastInterval;
  }, [ms, ...deps]);
}

export function useTimeout(cb: () => void, ms: number, deps: any[] = []) {
  const ref = useRef<number | null>(null);
  const clearLastTimeout = () => {
    if (ref.current) {
      clearTimeout(ref.current);
    }
  };
  useEffect(() => {
    clearLastTimeout();
    ref.current = window.setTimeout(cb, ms);
    return clearLastTimeout;
  }, [ms, ...deps]);
}

export function useSyncClockInterval(cb: () => void, on: 'minutes' | 'seconds', deps: any[] = []) {
  const ref = useRef<number | null>(null);

  const clearLastInterval = () => {
    if (ref.current) {
      clearInterval(ref.current);
    }
  };

  useEffect(() => {
    clearLastInterval();

    const now = new Date();
    let msToWaitForClockSync = 0;
    if (on === 'minutes') {
      const secondsUntilNextMinute = 60 - now.getSeconds();
      msToWaitForClockSync = secondsUntilNextMinute * 1000 - now.getMilliseconds();
    } else if (on === 'seconds') {
      msToWaitForClockSync = 1000 - now.getMilliseconds();
    }

    setTimeout(() => {
      cb();
      let interval = on === 'minutes' ? 60 * 1000 : 1000;
      ref.current = window.setInterval(cb, interval);
    }, msToWaitForClockSync);

    return clearLastInterval;
  }, [on, ...deps]);
}

export function useDeepCompareEffect(callback: () => void, dependencies: any[]) {
  const currentDependenciesRef = useRef<any[]>();
  if (!isEqual(currentDependenciesRef.current, dependencies)) {
    currentDependenciesRef.current = dependencies;
  }
  useEffect(callback, [currentDependenciesRef.current]);
}

export function useDebounce<F extends (...args: any[]) => void>(
  callback: F,
  ms: number,
): DebouncedFunc<F> {
  const ref = useRef<F>();

  useEffect(() => {
    ref.current = callback;
  }, [callback]);

  const debouncedCallback = useMemo(() => {
    const func = (...args: Parameters<F>) => {
      ref.current?.(...args);
    };
    return debounce(func, ms);
  }, []);

  return debouncedCallback;
}

export function useThrottle<F extends (...args: any[]) => void>(
  callback: F,
  ms: number,
  options?: ThrottleSettings,
): DebouncedFuncLeading<F> {
  const ref = useRef<F>();

  useEffect(() => {
    ref.current = callback;
  }, [callback]);

  const throttledCallback = useMemo(() => {
    const func = (...args: Parameters<F>) => {
      ref.current?.(...args);
    };
    return throttle(func, ms, options);
  }, []);

  return throttledCallback;
}
