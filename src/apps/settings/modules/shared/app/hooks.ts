import { useCallback } from 'react';
import { TypedUseSelectorHook, useDispatch, useSelector, useStore } from 'react-redux';

import type { AppDispatch, store } from '../infrastructure/store';

import { RootState } from '../domain/state';

type DispatchFunc = () => AppDispatch;
export const useAppDispatch: DispatchFunc = useDispatch;
export const useAppSelector: TypedUseSelectorHook<RootState> = useSelector;
export const useAppStore: () => store = useStore;

export const useDispatchCallback = <fn extends anyFunction>(cb: fn): fn => useCallback(cb, []);