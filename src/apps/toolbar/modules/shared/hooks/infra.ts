import { ExtraCallbacksOnBlur, ExtraCallbacksOnFocus } from '../../../events';
import { useEffect, useRef } from 'react';

export function useAppBlur(cb: () => void, deps: any[] = []) {
  const key = useRef(crypto.randomUUID());
  useEffect(() => {
    ExtraCallbacksOnBlur.remove(key.current);
    ExtraCallbacksOnBlur.add(cb, key.current);
    return () => {
      ExtraCallbacksOnBlur.remove(key.current);
    };
  }, deps);
}

export function useAppActivation(cb: () => void, deps: any[] = []) {
  const key = useRef(crypto.randomUUID());
  useEffect(() => {
    ExtraCallbacksOnFocus.remove(key.current);
    ExtraCallbacksOnFocus.add(cb, key.current);
    return () => {
      ExtraCallbacksOnFocus.remove(key.current);
    };
  }, deps);
}

export function useInterval(callback: () => void, delay: number, deps: any[] = []) {
  const key = useRef<number>();

  // Set up the interval.
  useEffect(() => {
    if (key.current) {
      clearInterval(key.current);
    }
    key.current = window.setInterval(callback, delay);
    return () => clearInterval(key.current);
  }, [callback, delay, ...deps]);
}