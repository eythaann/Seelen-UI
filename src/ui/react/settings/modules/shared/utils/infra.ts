import { useCallback } from "react";
import { type TypedUseSelectorHook, useDispatch, useSelector, useStore } from "react-redux";

import type { AppDispatch, store } from "../store/infra.ts";

import type { RootState } from "../store/domain.ts";

type DispatchFunc = () => AppDispatch;
export const useAppDispatch: DispatchFunc = useDispatch;
export const useAppSelector: TypedUseSelectorHook<RootState> = useSelector;
export const useAppStore: () => store = useStore;

export const useDispatchCallback = <fn extends anyFunction>(cb: fn): fn => useCallback(cb, []);
