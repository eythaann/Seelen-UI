import { Action, Slice } from '@reduxjs/toolkit';

import { HexColor, SelectorsFor } from '../domain/interfaces';

type Args = undefined | string | { [x: string]: boolean };
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

export const selectorsFor = <T>(state: T): SelectorsFor<T> => {
  const selectors = {} as SelectorsFor<T>;
  for (const key in state) {
    selectors[key] = (state: T) => state[key];
  }
  return selectors;
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