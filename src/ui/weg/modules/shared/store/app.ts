import { createSlice } from "@reduxjs/toolkit";
import { WegItemType } from "@seelen-ui/lib";
import { StateBuilder } from "@shared/StateBuilder";

import { PinnedWegItem, RootState, SwItem, TemporalWegItem } from "./domain";

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
  return item.type === WegItemType.Pinned;
};

export const isTemporalApp = (item: SwItem): item is TemporalWegItem => {
  return item.type === WegItemType.Temporal;
};
