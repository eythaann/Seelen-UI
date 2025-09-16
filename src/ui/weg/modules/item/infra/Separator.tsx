import { cx } from "@shared/styles";

import { SeparatorWegItem } from "../../shared/store/domain";

import { HardcodedSeparator1, HardcodedSeparator2 } from "../../shared/state/items";
import { $settings } from "../../shared/state/mod";

export function Separator({ item }: { item: SeparatorWegItem }) {
  return (
    <div
      className={cx("weg-separator", {
        "weg-separator-1": item.id === HardcodedSeparator1.id,
        "weg-separator-2": item.id === HardcodedSeparator2.id,
        visible: $settings.value.visibleSeparators,
      })}
    />
  );
}
