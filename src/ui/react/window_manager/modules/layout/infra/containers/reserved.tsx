import { cx } from "@shared/styles";

import { Reservation } from "../../domain.ts";

import { $settings } from "../../../shared/state/mod.ts";

export function ReservedContainer(
  { reservation }: { reservation: Reservation },
) {
  const { floating } = $settings.value;
  return (
    <div
      className={cx(
        "wm-container",
        "wm-reserved",
        `wm-reserved-${reservation.toLowerCase()}`,
      )}
      style={reservation === Reservation.Float
        ? {
          width: floating.width,
          height: floating.height,
        }
        : undefined}
    />
  );
}
