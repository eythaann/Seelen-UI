import { getCurrentWindow } from '@tauri-apps/api/window';
import { useEffect, useRef } from 'react';

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
