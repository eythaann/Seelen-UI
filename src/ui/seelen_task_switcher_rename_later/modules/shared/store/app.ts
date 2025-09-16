import { createSlice } from "@reduxjs/toolkit";
import { StateBuilder } from "@shared/StateBuilder";

import { Desktop, RootState } from "./domain";

const desk1 = new Desktop("1", "Desktop 1");
const desk2 = new Desktop("2", "Desktop 2");
const desk3 = new Desktop("3", "Desktop 3");
const desk4 = new Desktop("4", "Desktop 4");
const desk5 = new Desktop("5", "Desktop 5");
const desk6 = new Desktop("6", "Desktop 6");
const desk7 = new Desktop("7", "Desktop 7");
const desk8 = new Desktop("8", "Desktop 8");

/* desk1.linkTo(desk2, Placement.Top);
desk2.linkTo(desk3, Placement.Right);
desk3.linkTo(desk4, Placement.Bottom);
desk4.linkTo(desk1, Placement.Left);

desk1.linkTo(desk3, Placement.TopRight);
desk2.linkTo(desk4, Placement.BottomRight); */

const initialState: RootState = {
  desktops: [desk1, desk2, desk3, desk4, desk5, desk6, desk7, desk8],
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
