import { SeelenWegSlice } from "../../../seelenweg/app.ts";
import { BorderSlice } from "../../../WindowManager/border/app.ts";
import { SeelenManagerSlice } from "../../../WindowManager/main/app.ts";

import type { RootState } from "../domain.ts";

import { RootSlice } from "./reducer.ts";

export const ownSelector = (state: RootState) => state;
export const RootSelectors = RootSlice.getSelectors(ownSelector);

export const SeelenWegSelectors = SeelenWegSlice.getSelectors(
  RootSelectors.seelenweg,
);
export const SeelenWmSelectors = SeelenManagerSlice.getSelectors(
  RootSelectors.windowManager,
);

export const BorderSelectors = BorderSlice.getSelectors(
  SeelenWmSelectors.border,
);
