import { invoke, SeelenCommand, Widget } from "@seelen-ui/lib";
import type { ContextMenu } from "@seelen-ui/lib/types";

import { $actions } from "../../shared/state/items.ts";
import { useTranslation } from "react-i18next";
import { useCallback, useEffect, useMemo } from "preact/hooks";

const identifier = crypto.randomUUID();

export function useItemContextMenu(itemId: string) {
  const { t } = useTranslation();

  // Memoize callbackEvent since it only depends on itemId
  const callbackEvent = useMemo(
    () => `context-menu::${itemId.replace("@", "")}`,
    [itemId],
  );

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
  }, [itemId, callbackEvent]);

  // Memoize the menu object to prevent reconstruction on every render
  const menu: ContextMenu = useMemo(() => ({
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
  }), [t, callbackEvent]);

  // Memoize the callback to prevent recreation
  const onContextMenu = useCallback(() => {
    invoke(SeelenCommand.TriggerContextMenu, {
      menu,
      forwardTo: null,
    });
  }, [menu]);

  return {
    onContextMenu,
  };
}
