import { createSlice } from "@reduxjs/toolkit";
import type { WindowManagerSettings } from "@seelen-ui/lib/types";

import { defaultSettings } from "../../shared/store/app/default.ts";
import { matcher, reducersFor, selectorsFor } from "../../shared/utils/app.ts";
import { BorderSlice } from "../border/app.ts";

let initialState: WindowManagerSettings = defaultSettings.windowManager;

export const SeelenManagerSlice = createSlice({
  name: "windowManager",
  initialState,
  selectors: selectorsFor(initialState),
  reducers: {
    ...reducersFor(initialState),
  },
  extraReducers: (builder) => {
    builder
      .addMatcher(matcher(BorderSlice), (state, action) => {
        state.border = BorderSlice.reducer(state.border, action);
      });
  },
});

export const WManagerSettingsActions = SeelenManagerSlice.actions;
