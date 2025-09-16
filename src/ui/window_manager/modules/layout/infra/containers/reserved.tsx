import { cx } from "@shared/styles";

import { Reservation } from "../../domain";

import { $settings } from "../../../shared/state/mod";

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
