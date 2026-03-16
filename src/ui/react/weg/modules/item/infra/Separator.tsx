import { cx } from "libs/ui/react/utils/styling.ts";

import type { SeparatorWegItem } from "../../shared/types.ts";

import { HARDCODED_SEPARATOR_LEFT, HARDCODED_SEPARATOR_RIGHT } from "../../shared/state/items.ts";
import { $settings } from "../../shared/state/mod.ts";

export function Separator({ item }: { item: SeparatorWegItem }) {
  return (
    <div
      className={cx("weg-separator", {
        "weg-separator-1": item.id === HARDCODED_SEPARATOR_LEFT.id,
        "weg-separator-2": item.id === HARDCODED_SEPARATOR_RIGHT.id,
        visible: $settings.value.visibleSeparators,
      })}
    />
  );
}
