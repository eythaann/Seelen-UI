import { createSlice } from "@reduxjs/toolkit";
import type { FancyToolbarSettings } from "@seelen-ui/lib/types";

import { defaultSettings } from "../shared/store/app/default.ts";
import { reducersFor } from "../shared/utils/app.ts";

const initialState: FancyToolbarSettings = defaultSettings.fancyToolbar;

export const FancyToolbarSlice = createSlice({
  name: "fancyToolbar",
  initialState,
  reducers: reducersFor(initialState),
});

export const FancyToolbarActions = FancyToolbarSlice.actions;
