import { createSlice } from "@reduxjs/toolkit";
import { FancyToolbarSettings } from "@seelen-ui/lib/types";

import { defaultSettings } from "../shared/store/app/default";
import { reducersFor } from "../shared/utils/app";

const initialState: FancyToolbarSettings = defaultSettings.fancyToolbar;

export const FancyToolbarSlice = createSlice({
  name: "fancyToolbar",
  initialState,
  reducers: reducersFor(initialState),
});

export const FancyToolbarActions = FancyToolbarSlice.actions;
