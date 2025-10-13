import { cx } from "@shared/styles";

import type { SeparatorWegItem } from "../../shared/store/domain.ts";

import { HardcodedSeparator1, HardcodedSeparator2 } from "../../shared/state/items.ts";
import { $settings } from "../../shared/state/mod.ts";

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
