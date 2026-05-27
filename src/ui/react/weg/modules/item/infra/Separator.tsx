import { invoke, SeelenCommand, Widget } from "@seelen-ui/lib";
import { cx } from "libs/ui/react/utils/styling.ts";
import { useCallback } from "react";
import { useTranslation } from "react-i18next";

import type { SeparatorWegItem } from "../../shared/types.ts";

import { $dock_state_actions, HARDCODED_SEPARATOR_LEFT, HARDCODED_SEPARATOR_RIGHT } from "../../shared/state/items.ts";
import { $settings, getDockContextMenuAlignment } from "../../shared/state/settings.ts";

const identifier = crypto.randomUUID();
const onSeparatorMenuClick = "weg::separator_menu_click";

Widget.self.webview.listen(onSeparatorMenuClick, ({ payload }) => {
  const { key } = payload as { key: string };
  if (key === "create_group") {
    $dock_state_actions.createFolder();
  }
});

export function Separator({ item }: { item: SeparatorWegItem }) {
  const { t } = useTranslation();

  const onContextMenu = useCallback(
    (e: MouseEvent) => {
      e.stopPropagation();
      const { alignX, alignY } = getDockContextMenuAlignment($settings.value.position);
      invoke(SeelenCommand.TriggerContextMenu, {
        menu: {
          identifier,
          alignX,
          alignY,
          items: [
            {
              type: "Item",
              key: "create_group",
              icon: "MdCreateNewFolder",
              label: t("separator.create_group", "Create Group"),
              callbackEvent: onSeparatorMenuClick,
            },
          ],
        },
        forwardTo: null,
      });
    },
    [t],
  );

  return (
    <div
      className={cx("weg-separator", {
        "weg-separator-1": item.id === HARDCODED_SEPARATOR_LEFT.id,
        "weg-separator-2": item.id === HARDCODED_SEPARATOR_RIGHT.id,
        visible: $settings.value.visibleSeparators,
      })}
      onContextMenu={onContextMenu}
    />
  );
}
