import { listen } from "@tauri-apps/api/event";
import { invoke, SeelenCommand, SeelenEvent, subscribe } from "@seelen-ui/lib";
import type { SysTrayIcon, SysTrayIconId, ToolbarItem, ToolbarItem2 } from "@seelen-ui/lib/types";
import { useEffect, useState } from "preact/hooks";
import { baseItem } from "../shared/state/default.ts";
import { $toolbar_dragging, $toolbar_state } from "../shared/state/items.ts";

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

/**
 * Stable identity used for a pinned icon's toolbar item id.
 *
 * IMPORTANT: this must NOT depend on the tooltip, because many tray icons
 * (CPU/GPU temperature, battery, network, etc.) update their tooltip and icon
 * image every second. Deriving the id from the tooltip made the item id change
 * constantly, so the sync effect kept removing and re-inserting the item, which
 * is what caused the pinned icons to jump around.
 *
 * `guid` is stable across app/seelen restarts; the serialized `stable_id` is
 * stable for the lifetime of the window. We derive the id from the *stored*
 * pinned icon so it stays consistent even if the underlying window handle
 * changes (the live tray item is still matched via `matchesPinnedTrayIcon`).
 */
function stableKeyFromPinnedIcon(pinnedIcon: PinnedTrayIcon): string {
  return pinnedIcon.guid ? `guid::${pinnedIcon.guid}` : `sid::${pinnedIcon.key}`;
}

function toolbarIdFromStableKey(key: string): string {
  return `pinned-tray::${btoa(unescape(encodeURIComponent(key)))}`;
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

function createToolbarItem(item: SysTrayIcon, id: string): ToolbarItem {
  const trayIconId = JSON.stringify(item.stable_id);
  const tooltip = JSON.stringify(getItemName(item));

  // The template is STATIC (just the tray id): the icon is resolved live from
  // the shared tray state by `TrayIcon`. This keeps the toolbar item unchanged
  // across icon updates — so it never churns/persists the toolbar state (which
  // made icons jump, especially while dragging) — while still updating live.
  return {
    ...baseItem,
    id,
    // `trayIconId` is already a JSON string; stringify again so it is embedded as
    // a string literal (TrayIcon expects the serialized id as a string).
    template: `return TrayIcon({ id: ${JSON.stringify(trayIconId)} });`,
    tooltip: `return ${tooltip};`,
    onClick: `invoke(SeelenCommand.SendSystemTrayIconAction, { id: ${trayIconId}, action: "LeftClick" });`,
    onDoubleClick: `invoke(SeelenCommand.SendSystemTrayIconAction, { id: ${trayIconId}, action: "LeftDoubleClick" });`,
    onContextMenu: `invoke(SeelenCommand.SendSystemTrayIconAction, { id: ${trayIconId}, action: "RightClick" });`,
    style: { flexShrink: 0 },
  } as ToolbarItem;
}

function isPinnedToolbarItem(item: unknown): item is ToolbarItem {
  return typeof item === "object" && item !== null && "id" in item &&
    typeof item.id === "string" && item.id.startsWith("pinned-tray::");
}

/**
 * Whether two pinned toolbar items render/behave the same. Only the icon image
 * and the action target matter here — we intentionally ignore tooltip-only
 * changes so a live tooltip doesn't trigger a state write every second.
 */
function pinnedItemsEqual(a: ToolbarItem, b: ToolbarItem) {
  return a.template === b.template && a.onClick === b.onClick;
}

export function PinnedTrayIcons() {
  const [pinnedIcons, setPinnedIcons] = useState<PinnedTrayIcon[]>([]);
  const [trayItems, setTrayItems] = useState<SysTrayIcon[]>([]);
  const dragging = $toolbar_dragging.value;

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

  // Reconcile pinned icons with the toolbar items. Positions live in the
  // (persisted) toolbar items array, so they stay fixed across restarts and can
  // be reordered by the user via drag & drop. We only ever:
  //   - add a brand new pinned item (at the system tray position),
  //   - remove an item the user actually unpinned,
  //   - refresh an existing item's icon in place (without moving it).
  useEffect(() => {
    // Don't touch the toolbar items while the user is dragging — reconciling
    // here (e.g. on a live tray-icon update) would fight the drag reorder and
    // make icons jump around.
    if (dragging) {
      return;
    }

    const toolbarItems = $toolbar_state.peek().items;

    const existingById = new Map<string, ToolbarItem>();
    for (const item of toolbarItems) {
      if (isPinnedToolbarItem(item)) {
        existingById.set(item.id, item);
      }
    }

    // Desired pinned items keyed by a stable id (regardless of visibility).
    const desired = new Map<string, ToolbarItem>();
    for (const pinnedIcon of pinnedIcons) {
      const id = toolbarIdFromStableKey(stableKeyFromPinnedIcon(pinnedIcon));
      const trayItem = trayItems.find((t) => matchesPinnedTrayIcon(t, pinnedIcon));
      if (trayItem) {
        desired.set(id, createToolbarItem(trayItem, id));
      } else if (existingById.has(id)) {
        // Keep the last known item so its slot/position survives a temporary
        // disappearance (overflow toggle, app briefly closing, etc.).
        desired.set(id, existingById.get(id)!);
      }
    }
    const desiredIds = new Set(desired.keys());

    let changed = false;
    const next: ToolbarItem2[] = [];
    for (const item of toolbarItems) {
      if (!isPinnedToolbarItem(item)) {
        next.push(item);
        continue;
      }
      if (!desiredIds.has(item.id)) {
        changed = true; // unpinned by the user
        continue;
      }
      const fresh = desired.get(item.id)!;
      if (!pinnedItemsEqual(fresh, item)) {
        changed = true; // icon/action changed → refresh in place (keep position)
        next.push(fresh);
      } else {
        next.push(item);
      }
    }

    // Append brand new pinned items at the system tray position.
    const present = new Set(
      next.filter(isPinnedToolbarItem).map((it) => it.id),
    );
    const missing: ToolbarItem[] = [];
    for (const [id, item] of desired) {
      if (!present.has(id)) {
        missing.push(item);
      }
    }
    if (missing.length) {
      changed = true;
      const systemTrayIndex = next.findIndex((item) => item === "@seelen/tb-system-tray");
      next.splice(systemTrayIndex === -1 ? next.length : systemTrayIndex, 0, ...missing);
    }

    if (changed) {
      $toolbar_state.value = {
        ...$toolbar_state.peek(),
        items: next,
      };
    }
  }, [pinnedIcons, trayItems, dragging]);

  // Enrich legacy pins (stored without identity data) once their tray item is
  // seen, so matching/ids stay stable afterwards.
  useEffect(() => {
    if (!pinnedIcons.length || !trayItems.length) {
      return;
    }

    let changed = false;
    const hydratedIcons = pinnedIcons.map((pinnedIcon) => {
      if (hasIdentityData(pinnedIcon)) {
        return pinnedIcon;
      }

      const item = trayItems.find((trayItem) => matchesPinnedTrayIcon(trayItem, pinnedIcon));
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

  return null;
}
