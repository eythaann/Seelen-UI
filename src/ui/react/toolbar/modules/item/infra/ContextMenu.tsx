import { invoke, SeelenCommand, Widget } from "@seelen-ui/lib";
import type { ContextMenu } from "@seelen-ui/lib/types";

import { $actions } from "../../shared/state/items.ts";
import { useTranslation } from "react-i18next";
import { useEffect } from "preact/hooks";

const identifier = crypto.randomUUID();

export function useItemContextMenu(itemId: string) {
  const { t } = useTranslation();

  const callbackEvent = `context-menu::${itemId.replace("@", "")}`;

  useEffect(() => {
    const unlistener = Widget.self.webview.listen(callbackEvent, ({ payload }) => {
      const { key } = payload as any;
      if (key === "remove") {
        $actions.removeItem(itemId);
      }
    });

    return () => {
      unlistener.then((cb) => cb());
    };
  }, [itemId]);

  const menu: ContextMenu = {
    identifier,
    items: [
      {
        type: "Item",
        key: "remove",
        label: t("context_menu.remove"),
        icon: "CgExtensionRemove",
        callbackEvent,
      },
    ],
  };

  return {
    onContextMenu: () => {
      invoke(SeelenCommand.TriggerContextMenu, {
        menu,
        forwardTo: null,
      });
    },
  };
}
