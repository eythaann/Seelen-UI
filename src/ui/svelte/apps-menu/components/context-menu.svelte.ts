import { invoke, SeelenCommand, SeelenEvent, Widget } from "@seelen-ui/lib";
import type { ContextMenu, ContextMenuCallbackPayload, StartMenuItem } from "@seelen-ui/lib/types";
import { type FavFolderItem, globalState } from "../state/mod.svelte";
import { emit } from "@tauri-apps/api/event";

export const CONTEXT_MENU_ID = crypto.randomUUID();
export const CONTEXT_MENU_CALLBACK_EVENT = "apps_menu::context_menu_action";

export const FOLDER_CONTEXT_MENU_ID = crypto.randomUUID();
const FOLDER_CONTEXT_MENU_CALLBACK_EVENT = "apps_menu::folder_context_menu_action";

Widget.self.webview.listen<ContextMenuCallbackPayload>("item::context-menu", ({ payload }) => {
  const { key, meta } = payload;
  const item = (meta as any).item as StartMenuItem;

  if (key === "pin") {
    globalState.togglePin(item);
  } else if (key === "open_file_location") {
    Widget.self.hide();
    invoke(SeelenCommand.SelectFileOnExplorer, { path: item.path });
  } else if (key === "pin_to_dock") {
    emit(SeelenEvent.WegAddItem, {
      id: crypto.randomUUID(),
      displayName: item.display_name,
      umid: item.umid,
      path: item.path,
      pinned: true,
      preventPinning: false,
      relaunch: null,
    });
  } else if (key === "run_as_admin") {
    Widget.self.hide();
    const program = item.umid ? `shell:AppsFolder\\${item.umid}` : item.path;
    invoke(SeelenCommand.Run, { program, args: null, workingDir: null, elevated: true });
  }
});

Widget.self.webview.listen<ContextMenuCallbackPayload>(
  FOLDER_CONTEXT_MENU_CALLBACK_EVENT,
  ({ payload }) => {
    const { key, meta } = payload;
    const folderId = (meta as any).folderId as string;

    if (key === "disband") {
      globalState.disbandFolder(folderId);
    }
  },
);

export function getFolderContextMenu(
  folder: FavFolderItem,
  t: (key: string) => string,
): ContextMenu {
  return {
    identifier: FOLDER_CONTEXT_MENU_ID,
    meta: { folderId: folder.itemId },
    items: [
      {
        type: "Item",
        key: "disband",
        label: t("disband"),
        icon: "GiExpand",
        callbackEvent: FOLDER_CONTEXT_MENU_CALLBACK_EVENT,
      },
    ],
  };
}

export function getItemContextMenu(item: StartMenuItem, t: (key: string) => string): ContextMenu {
  const isPinned = globalState.isPinned(item);

  return {
    identifier: CONTEXT_MENU_ID,
    meta: { item },
    items: [
      {
        type: "Item",
        key: "pin",
        label: isPinned ? t("unpin") : t("pin"),
        icon: isPinned ? "TbPinnedOff" : "TbPin",
        callbackEvent: CONTEXT_MENU_CALLBACK_EVENT,
      },
      ...(item.path
        ? [
          {
            type: "Item" as const,
            key: "open_file_location",
            label: t("open_file_location"),
            icon: "MdOutlineMyLocation",
            callbackEvent: CONTEXT_MENU_CALLBACK_EVENT,
          },
        ]
        : []),
      {
        type: "Item",
        key: "pin_to_dock",
        label: t("pin_to_dock"),
        icon: "RiPushpinLine",
        callbackEvent: CONTEXT_MENU_CALLBACK_EVENT,
      },
      ...(item.umid || item.path.toLowerCase().endsWith(".lnk")
        ? [
          {
            type: "Item" as const,
            key: "run_as_admin",
            label: t("run_as_admin"),
            icon: "MdOutlineAdminPanelSettings",
            callbackEvent: CONTEXT_MENU_CALLBACK_EVENT,
          },
        ]
        : []),
    ],
  };
}
