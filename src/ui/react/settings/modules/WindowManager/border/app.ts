import { createSlice } from "@reduxjs/toolkit";
import type { Border } from "@seelen-ui/lib/types";

import { reducersFor, selectorsFor } from "../../shared/utils/app.ts";

const initialState: Border = {
  enabled: false,
  offset: 0,
  width: 2,
};

export const BorderSlice = createSlice({
  name: "windowManager/border",
  initialState,
  reducers: reducersFor(initialState),
  selectors: selectorsFor(initialState),
});

export const BorderActions = BorderSlice.actions;
