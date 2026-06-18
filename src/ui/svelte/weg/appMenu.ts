import { invoke, SeelenCommand, Widget } from "@seelen-ui/lib";
import type { ContextMenu, ContextMenuItem, UserAppWindow, WidgetId } from "@seelen-ui/lib/types";
import type { AppOrFileWegItem } from "./types.ts";
import { dockStateActions } from "./state/items.svelte.ts";
import { fullSettings } from "./state/settings.svelte.ts";
import { iconPackManager } from "libs/ui/svelte/components/Icon/index.ts";
import { prefersDarkColorScheme } from "libs/ui/svelte/runes/DarkMode.svelte.ts";

const identifier = crypto.randomUUID();
const onAppMenuClick = "weg::app_menu_click";

let pendingAppItem: AppOrFileWegItem | null = null;
let pendingAppWindows: UserAppWindow[] = [];

Widget.self.webview.listen(onAppMenuClick, ({ payload }) => {
  const { key } = payload as { key: string };
  const item = pendingAppItem;
  const windows = pendingAppWindows;
  if (!item) return;

  if (key === "unpin") {
    if (windows.length) {
      dockStateActions.unpinApp(item.id);
    } else {
      dockStateActions.remove(item.id);
    }
  } else if (key === "pin") {
    dockStateActions.pinApp(item.id);
  } else if (key === "run") {
    launchItem(item, false);
  } else if (key === "open_location") {
    invoke(SeelenCommand.SelectFileOnExplorer, { path: item.path });
  } else if (key === "run_as") {
    launchItem(item, true);
  } else if (key === "copy_hwnd") {
    navigator.clipboard.writeText(JSON.stringify(windows.map((w) => w.hwnd.toString(16))));
  } else if (key === "close") {
    windows.forEach((w) => invoke(SeelenCommand.WegCloseApp, { hwnd: w.hwnd }));
  } else if (key === "kill") {
    windows.forEach((w) => invoke(SeelenCommand.WegKillApp, { hwnd: w.hwnd }));
  } else if (key === "edit_app_icon") {
    const entry = iconPackManager.value.getIconEntry({ path: item.path, umid: item.umid });
    invoke(SeelenCommand.TriggerWidget, {
      payload: {
        id: "@seelen/icon-editor" as WidgetId,
        customArgs: { entry },
      },
    });
  }
});

export function getUserApplicationContextMenu(
  t: (key: string) => string,
  item: AppOrFileWegItem,
  windows: UserAppWindow[],
): ContextMenu {
  pendingAppItem = item;
  pendingAppWindows = windows;

  const items: ContextMenuItem[] = [];

  if (!item.preventPinning) {
    if (item.pinned) {
      items.push({
        type: "Item",
        key: "unpin",
        icon: "RiUnpinLine",
        label: t("app_menu.unpin"),
        callbackEvent: onAppMenuClick,
      });
    } else {
      items.push({
        type: "Item",
        key: "pin",
        icon: "RiPushpinLine",
        label: t("app_menu.pin"),
        callbackEvent: onAppMenuClick,
      });
    }
    items.push({ type: "Separator" });
  }

  const foundIcon = iconPackManager.value.getIcon({ path: item.path, umid: item.umid });
  const iconSrc = (prefersDarkColorScheme.value ? foundIcon?.dark : foundIcon?.light) || foundIcon?.base;

  items.push(
    {
      type: "Item",
      key: "run",
      icon: iconSrc ?? "IoOpenOutline",
      label: item.displayName,
      callbackEvent: onAppMenuClick,
    },
    {
      type: "Item",
      key: "open_location",
      icon: "MdOutlineMyLocation",
      label: t("app_menu.open_file_location"),
      callbackEvent: onAppMenuClick,
    },
    {
      type: "Item",
      key: "run_as",
      icon: "MdOutlineAdminPanelSettings",
      label: t("app_menu.run_as"),
      callbackEvent: onAppMenuClick,
    },
    {
      type: "Item",
      key: "edit_app_icon",
      icon: "RiEditBoxLine",
      label: t("app_menu.edit_app_icon"),
      callbackEvent: onAppMenuClick,
    },
  );

  if (windows.length) {
    items.push({ type: "Separator" });

    if (fullSettings.value.devTools) {
      items.push({
        type: "Item",
        key: "copy_hwnd",
        icon: "AiOutlineCopy",
        label: t("app_menu.copy_handles"),
        callbackEvent: onAppMenuClick,
      });
    }

    items.push({
      type: "Item",
      key: "close",
      icon: "BiWindowClose",
      label: windows.length > 1 ? t("app_menu.close_multiple") : t("app_menu.close"),
      callbackEvent: onAppMenuClick,
      danger: true,
    });

    const settings = fullSettings.value.byWidget["@seelen/weg"] as any;
    if (settings?.showEndTask) {
      items.push({
        type: "Item",
        key: "kill",
        icon: "MdOutlineDangerous",
        label: windows.length > 1 ? t("app_menu.kill_multiple") : t("app_menu.kill"),
        callbackEvent: onAppMenuClick,
        danger: true,
      });
    }
  }

  return { identifier, items };
}

export function launchItem(item: AppOrFileWegItem, elevated: boolean) {
  if (item.relaunch) {
    return invoke(SeelenCommand.Run, {
      program: item.relaunch.command,
      args: item.relaunch.args,
      workingDir: item.relaunch.workingDir,
      elevated,
    });
  }

  return invoke(SeelenCommand.Run, {
    program: item.umid ? `shell:AppsFolder\\${item.umid}` : item.path,
    args: null,
    workingDir: null,
    elevated,
  });
}
