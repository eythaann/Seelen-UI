import { Action, Slice } from '@reduxjs/toolkit';

import { HexColor, ReducersFor, SelectorsFor } from '../domain/interfaces';

type Args = undefined | string | { [x: string]: boolean | null | undefined };
export const cx = (...args: Args[]): string => {
  return args.map((arg) => {
    if (!arg) {
      return;
    }

    if (typeof arg === 'string') {
      return arg;
    }

    return Object.keys(arg).map((key) => arg[key] ? key : '').join(' ');
  }).join(' ');
};

export const matcher = (slice: Slice) => (action: Action) => action.type.startsWith(slice.name);

export const selectorsFor = <T extends anyObject>(state: T): SelectorsFor<T> => {
  const selectors = {} as SelectorsFor<T>;
  for (const key in state) {
    selectors[key] = (state: T) => state[key];
  }
  return selectors;
};

export const capitalize = (text: string) => {
  return text.slice(0, 1).toUpperCase() + text.slice(1);
};

export const reducersFor = <T>(state: T): ReducersFor<T> => {
  const reducers: any = {};
  for (const key in state) {
    reducers[`set${capitalize(key)}`] = (state: T, action: any) => {
      state[key] = action.payload;
    };
  }
  return reducers;
};

export const defaultOnNull = <T>(value: T | null | undefined, onNull: T): T => {
  if (value == null) {
    return onNull;
  }
  return value;
};

export const validateHexColor = (str: string): HexColor | null => {
  if (!str.startsWith('#')) {
    return null;
  }
  return str as HexColor;
};

export const OptionsFromEnum = (obj: anyObject) => Object.values(obj).map((value) => ({
  label: value,
  value,
}));

export function debounce<T extends anyFunction>(fn: T, delay: number): T {
  let timeoutId: NodeJS.Timeout;

  return function debounced(this: ThisParameterType<T>, ...args: Parameters<T>): void {
    const context = this;
    clearTimeout(timeoutId);
    timeoutId = setTimeout(() => {
      fn.apply(context, args);
    }, delay);
  } as T;
}
