import { createSlice } from "@reduxjs/toolkit";
import { StateBuilder } from "@shared/StateBuilder";

import type { PinnedWegItem, RootState, SwItem, TemporalWegItem } from "./domain.ts";

const initialState: RootState = {
  devTools: false,
  focusedApp: null,
  mediaSessions: [],
  notifications: [],
};

export const RootSlice = createSlice({
  name: "root",
  initialState,
  reducers: {
    ...StateBuilder.reducersFor(initialState),
  },
});

export const RootActions = RootSlice.actions;
export const Selectors = StateBuilder.compositeSelector(initialState);

export const isPinnedApp = (item: SwItem): item is PinnedWegItem => {
  return item.type === "Pinned";
};

export const isTemporalApp = (item: SwItem): item is TemporalWegItem => {
  return item.type === "Temporal";
};
