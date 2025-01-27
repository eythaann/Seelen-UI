import { GetIconArgs, IconPackManager } from '@seelen-ui/lib';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { useEffect, useLayoutEffect, useRef, useState } from 'react';

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

const iconPackManager = await IconPackManager.create();
export function useIcon(args: GetIconArgs): string | null {
  const [iconSrc, setIconSrc] = useState<string | null>(() => iconPackManager.getIcon(args));

  useEffect(() => {
    iconPackManager.onChange(() => setIconSrc(iconPackManager.getIcon(args)));
  }, []);

  useLayoutEffect(() => {
    if (!iconSrc) {
      // this will run asynchronously on end `iconPackManager.onChange` will be triggered
      IconPackManager.extractIcon(args);
    }
  }, []);

  return iconSrc;
}
