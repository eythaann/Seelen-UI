import { CaseReducer, PayloadAction } from '@reduxjs/toolkit';
import { cast } from 'readable-types';

export type HexColor = `#${string}`;

export type SelectorsFor<T extends anyObject> = { [K in keyof T]: (state: T) => T[K] };

export type ReducersFor<T> = { [K in keyof T as `set${Capitalize<cast<K, string>>}`]: CaseReducer<T, PayloadAction<T[K]>> };