import { TwmReservation } from "@seelen-ui/lib/types";
import type { IconName } from "libs/ui/icons";

export const ReservationIcon: Record<TwmReservation, IconName> = {
  [TwmReservation.Left]: "TbBoxAlignLeft",
  [TwmReservation.Right]: "TbBoxAlignRight",
  [TwmReservation.Top]: "TbBoxAlignTop",
  [TwmReservation.Bottom]: "TbBoxAlignBottom",
  [TwmReservation.Stack]: "TbStack",
  [TwmReservation.Float]: "TbBoxMargin",
};

export const TREE_CONTEXT_KEY = "wm-tree";
