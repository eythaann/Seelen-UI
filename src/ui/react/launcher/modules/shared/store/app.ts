import { createSlice } from "@reduxjs/toolkit";
import { Settings, UIColors } from "@seelen-ui/lib";
import { StateBuilder } from "libs/ui/react/utils/StateBuilder.ts";

import type { LauncherState } from "./domain.ts";

const initialState: LauncherState = {
  colors: UIColors.default().inner,
  apps: [],
  history: {},
  settings: (await Settings.default()).launcher,
};

export const RootSlice = createSlice({
  name: "root",
  initialState,
  reducers: {
    ...StateBuilder.reducersFor(initialState),
  },
});

export const Actions = RootSlice.actions;
export const Selectors = StateBuilder.compositeSelector(initialState);
