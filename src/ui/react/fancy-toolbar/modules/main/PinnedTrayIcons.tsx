import { listen } from "@tauri-apps/api/event";
import { invoke, SeelenCommand, SeelenEvent, subscribe } from "@seelen-ui/lib";
import type { SysTrayIcon, SysTrayIconId, ToolbarItem } from "@seelen-ui/lib/types";
import { useComputed } from "@preact/signals";
import { useEffect, useMemo, useState } from "preact/hooks";
import { baseItem } from "../shared/state/default.ts";
import { $toolbar_state } from "../shared/state/items.ts";

const PINNED_TRAY_STORAGE_KEY = "seelen:pinned-tray-icons";
const PINNED_TRAY_CHANGED_EVENT = "seelen:pinned-tray-icons-changed";
const GET_PINNED_TRAY_ICONS_COMMAND = "get_pinned_tray_icons";
const SET_PINNED_TRAY_ICONS_COMMAND = "set_pinned_tray_icons";

type PinnedTrayIcon = {
  key: string;
  stableId: SysTrayIconId;
  tooltip: string;
  guid: string | null;
  uid: number | null;
};

function trayIdKey(id: SysTrayIconId) {
  return JSON.stringify(id);
}

function normalizePinnedTrayIcon(value: unknown): PinnedTrayIcon | null {
  if (typeof value === "string") {
    try {
      return {
        key: value,
        stableId: JSON.parse(value) as SysTrayIconId,
        tooltip: "",
        guid: null,
        uid: null,
      };
    } catch {
      return null;
    }
  }

  if (typeof value === "object" && value !== null && "key" in value && "stableId" in value) {
    return value as PinnedTrayIcon;
  }

  return null;
}

function getPinnedTrayIcons() {
  try {
    return (JSON.parse(localStorage.getItem(PINNED_TRAY_STORAGE_KEY) || "[]") as unknown[])
      .map(normalizePinnedTrayIcon)
      .filter((entry): entry is PinnedTrayIcon => !!entry);
  } catch {
    return [];
  }
}

async function getStoredPinnedTrayIcons() {
  try {
    const icons = await invoke(GET_PINNED_TRAY_ICONS_COMMAND as any) as PinnedTrayIcon[];
    if (icons.length) {
      return icons;
    }
  } catch {
    // Fall back to the old frontend storage while migrating existing pins.
  }

  return getPinnedTrayIcons();
}

function getItemName(item: SysTrayIcon) {
  return item.tooltip || item.guid || `${item.window_handle?.toString(16)}::${item.uid}`;
}

function getPinnedToolbarItemId(item: SysTrayIcon) {
  return `pinned-tray::${btoa(unescape(encodeURIComponent(getItemName(item))))}`;
}

function getPinnedToolbarItemIdFromPinnedIcon(pinnedIcon: PinnedTrayIcon) {
  const name = pinnedIcon.tooltip || pinnedIcon.guid || pinnedIcon.key;
  return `pinned-tray::${btoa(unescape(encodeURIComponent(name)))}`;
}

function toPinnedTrayIcon(item: SysTrayIcon): PinnedTrayIcon {
  return {
    key: trayIdKey(item.stable_id),
    stableId: item.stable_id,
    tooltip: item.tooltip,
    guid: item.guid,
    uid: item.uid,
  };
}

function matchesPinnedTrayIcon(item: SysTrayIcon, pinnedIcon: PinnedTrayIcon) {
  return trayIdKey(item.stable_id) === pinnedIcon.key ||
    (!!item.guid && item.guid === pinnedIcon.guid) ||
    (!!item.tooltip && item.tooltip === pinnedIcon.tooltip);
}

function hasIdentityData(pinnedIcon: PinnedTrayIcon) {
  return !!pinnedIcon.tooltip || !!pinnedIcon.guid || pinnedIcon.uid !== null;
}

async function setStoredPinnedTrayIcons(icons: PinnedTrayIcon[]) {
  localStorage.setItem(PINNED_TRAY_STORAGE_KEY, JSON.stringify(icons));
  await (invoke as any)(SET_PINNED_TRAY_ICONS_COMMAND, { pinnedIcons: icons });
}

function createToolbarItem(item: SysTrayIcon): ToolbarItem {
  const id = getPinnedToolbarItemId(item);
  const trayIconId = JSON.stringify(item.stable_id);
  const iconPath = JSON.stringify(item.icon_path);
  const iconHash = JSON.stringify(item.icon_image_hash || "null");
  const tooltip = JSON.stringify(getItemName(item));

  return {
    ...baseItem,
    id,
    template: item.icon_path ? `return Image({ path: ${iconPath}, url: null, hash: ${iconHash} });` : 'return ""',
    tooltip: `return ${tooltip};`,
    onClick: `invoke(SeelenCommand.SendSystemTrayIconAction, { id: ${trayIconId}, action: "LeftClick" });`,
    onContextMenu: `invoke(SeelenCommand.SendSystemTrayIconAction, { id: ${trayIconId}, action: "RightClick" });`,
    style: { flexShrink: 0 },
  } as ToolbarItem;
}

function isPinnedToolbarItem(item: unknown) {
  return typeof item === "object" && item !== null && "id" in item &&
    typeof item.id === "string" && item.id.startsWith("pinned-tray::");
}

export function PinnedTrayIcons() {
  const [pinnedIcons, setPinnedIcons] = useState<PinnedTrayIcon[]>([]);
  const [trayItems, setTrayItems] = useState<SysTrayIcon[]>([]);
  const pinnedToolbarOrder = useComputed(() =>
    $toolbar_state.value.items
      .filter(isPinnedToolbarItem)
      .map((item) => (item as ToolbarItem).id)
      .join("|")
  );

  useEffect(() => {
    let disposed = false;
    const unlisteners: Array<() => void> = [];
    const timeouts: Array<ReturnType<typeof setTimeout>> = [];

    const refreshPinnedIcons = () => {
      getStoredPinnedTrayIcons().then((icons) => {
        if (!disposed) {
          setPinnedIcons(icons);
        }
      });
    };

    invoke(SeelenCommand.GetSystemTrayIcons).then((items) => {
      if (!disposed) {
        setTrayItems(items);
      }
    });

    refreshPinnedIcons();
    timeouts.push(
      setTimeout(refreshPinnedIcons, 1000),
      setTimeout(refreshPinnedIcons, 3000),
      setTimeout(refreshPinnedIcons, 6000),
    );

    subscribe(SeelenEvent.SystemTrayChanged, ({ payload }) => {
      setTrayItems(payload);
      refreshPinnedIcons();
    }).then((unlisten) => unlisteners.push(unlisten));

    listen<unknown[]>(PINNED_TRAY_CHANGED_EVENT, ({ payload }) => {
      setPinnedIcons(
        payload
          .map(normalizePinnedTrayIcon)
          .filter((entry): entry is PinnedTrayIcon => !!entry),
      );
    }).then((unlisten) => unlisteners.push(unlisten));

    return () => {
      disposed = true;
      timeouts.forEach(clearTimeout);
      unlisteners.forEach((unlisten) => unlisten());
    };
  }, []);

  const pinnedItems = useMemo(() => {
    return pinnedIcons
      .map((pinnedIcon) => trayItems.find((item) => item.is_visible && matchesPinnedTrayIcon(item, pinnedIcon)))
      .filter((item): item is SysTrayIcon => !!item);
  }, [pinnedIcons, trayItems]);

  useEffect(() => {
    const toolbarItems = $toolbar_state.peek().items;
    const pinnedToolbarItems = pinnedItems.map(createToolbarItem);
    const pinnedToolbarIds = new Set(pinnedToolbarItems.map((item) => item.id));

    let changed = false;
    const nextItems = toolbarItems
      .filter((item) => !isPinnedToolbarItem(item) || pinnedToolbarIds.has((item as ToolbarItem).id));

    const existingIds = new Set(
      nextItems
        .filter(isPinnedToolbarItem)
        .map((item) => (item as ToolbarItem).id),
    );
    const missingItems = pinnedToolbarItems.filter((item) => !existingIds.has(item.id));

    if (missingItems.length) {
      changed = true;
      const systemTrayIndex = nextItems.findIndex((item) => item === "@seelen/tb-system-tray");
      nextItems.splice(systemTrayIndex === -1 ? nextItems.length : systemTrayIndex, 0, ...missingItems);
    }

    if (changed || nextItems.length !== toolbarItems.length) {
      $toolbar_state.value = {
        ...$toolbar_state.peek(),
        items: nextItems,
      };
    }
  }, [pinnedItems]);

  useEffect(() => {
    if (!pinnedIcons.length || !trayItems.length) {
      return;
    }

    let changed = false;
    const hydratedIcons = pinnedIcons.map((pinnedIcon) => {
      if (hasIdentityData(pinnedIcon)) {
        return pinnedIcon;
      }

      const item = trayItems.find((trayItem) => trayItem.is_visible && matchesPinnedTrayIcon(trayItem, pinnedIcon));
      if (!item) {
        return pinnedIcon;
      }

      changed = true;
      return toPinnedTrayIcon(item);
    });

    if (changed) {
      setPinnedIcons(hydratedIcons);
      setStoredPinnedTrayIcons(hydratedIcons).catch(console.error);
    }
  }, [pinnedIcons, trayItems]);

  useEffect(() => {
    if (!pinnedIcons.length || !pinnedToolbarOrder.value) {
      return;
    }

    const order = pinnedToolbarOrder.value.split("|");
    const orderIndex = new Map(order.map((id, index) => [id, index]));
    const orderedIcons = [...pinnedIcons].sort((a, b) => {
      const aIndex = orderIndex.get(getPinnedToolbarItemIdFromPinnedIcon(a)) ?? Number.MAX_SAFE_INTEGER;
      const bIndex = orderIndex.get(getPinnedToolbarItemIdFromPinnedIcon(b)) ?? Number.MAX_SAFE_INTEGER;
      return aIndex - bIndex;
    });

    if (JSON.stringify(orderedIcons) !== JSON.stringify(pinnedIcons)) {
      setStoredPinnedTrayIcons(orderedIcons).catch(console.error);
    }
  }, [pinnedIcons, pinnedToolbarOrder.value]);

  return null;
}
