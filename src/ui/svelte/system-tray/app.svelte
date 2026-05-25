<script lang="ts">
  import { SystrayIconAction, type SysTrayIcon, type SysTrayIconId } from "@seelen-ui/lib/types";
  import { state as trayState } from "./state.svelte";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { emit } from "@tauri-apps/api/event";
  import { invoke, SeelenCommand, Widget } from "@seelen-ui/lib";
  import { Icon, MissingIcon } from "libs/ui/svelte/components/Icon";

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

  let pinnedTrayIcons = $state<PinnedTrayIcon[]>([]);

  $effect(() => {
    Widget.getCurrent().ready();
  });

  getPinnedTrayIcons().then((icons) => {
    pinnedTrayIcons = icons;
  });

  function onClick(event: MouseEvent, id: SysTrayIconId) {
    // prevent be triggered by double click
    if (event.detail === 2) {
      return;
    }

    let action = SystrayIconAction.LeftClick;

    if (event.button === 1) {
      action = SystrayIconAction.MiddleClick;
    } else if (event.button === 2) {
      action = SystrayIconAction.RightClick;
    }

    invoke(SeelenCommand.SendSystemTrayIconAction, {
      id,
      action,
    });
  }

  function onDoubleClick(e: MouseEvent, id: SysTrayIconId) {
    e.preventDefault();
    e.stopPropagation();
    invoke(SeelenCommand.SendSystemTrayIconAction, {
      id,
      action: SystrayIconAction.LeftDoubleClick,
    });
  }

  function getItemName(item: SysTrayIcon) {
    return item.tooltip || item.guid || `${item.window_handle?.toString(16)}::${item.uid}`;
  }

  function trayIdKey(id: SysTrayIconId) {
    return JSON.stringify(id);
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

  function getLocalPinnedTrayIcons() {
    try {
      return (JSON.parse(localStorage.getItem(PINNED_TRAY_STORAGE_KEY) || "[]") as unknown[])
        .map(normalizePinnedTrayIcon)
        .filter((entry): entry is PinnedTrayIcon => !!entry);
    } catch {
      return [];
    }
  }

  async function getPinnedTrayIcons() {
    try {
      const icons = await (invoke as any)(GET_PINNED_TRAY_ICONS_COMMAND) as PinnedTrayIcon[];
      if (icons.length) {
        return icons;
      }
    } catch {
      // Fall back to the old frontend storage while migrating existing pins.
    }

    const localIcons = getLocalPinnedTrayIcons();
    if (localIcons.length) {
      await setPinnedTrayIcons(localIcons);
    }
    return localIcons;
  }

  async function setPinnedTrayIcons(icons: PinnedTrayIcon[]) {
    pinnedTrayIcons = icons;
    localStorage.setItem(PINNED_TRAY_STORAGE_KEY, JSON.stringify(icons));
    await (invoke as any)(SET_PINNED_TRAY_ICONS_COMMAND, { pinnedIcons: icons });
    emit(PINNED_TRAY_CHANGED_EVENT, icons);
  }

  function matchesPinnedTrayIcon(item: SysTrayIcon, pinnedIcon: PinnedTrayIcon) {
    return trayIdKey(item.stable_id) === pinnedIcon.key ||
      (!!item.guid && item.guid === pinnedIcon.guid) ||
      (!!item.tooltip && item.tooltip === pinnedIcon.tooltip);
  }

  function isPinned(item: SysTrayIcon) {
    return pinnedTrayIcons.some((pinnedIcon) => matchesPinnedTrayIcon(item, pinnedIcon));
  }

  async function onPinClick(event: Event, item: SysTrayIcon) {
    event.preventDefault();
    event.stopPropagation();

    const pinnedIcons = await getPinnedTrayIcons();
    const exists = pinnedIcons.some((pinnedIcon) => matchesPinnedTrayIcon(item, pinnedIcon));
    await setPinnedTrayIcons(
      exists
        ? pinnedIcons.filter((pinnedIcon) => !matchesPinnedTrayIcon(item, pinnedIcon))
        : [...pinnedIcons, toPinnedTrayIcon(item)],
    );
  }

  const GUIDS_TO_IGNORE = [
    "7820ae73-23e3-4229-82c1-e41cb67d5b9c", // speaker volument icon
    "7820ae74-23e3-4229-82c1-e41cb67d5b9c", // network icon
    "7820ae75-23e3-4229-82c1-e41cb67d5b9c", // battery icon
  ];
</script>

<div class={["slu-std-popover", "system-tray"]}>
  {#each trayState.trayItems as item}
    {#if item.is_visible && (!item.guid || !GUIDS_TO_IGNORE.includes(item.guid))}
      <button
        class="system-tray-item"
        onclick={(e) => onClick(e, item.stable_id)}
        ondblclick={(e) => onDoubleClick(e, item.stable_id)}
        oncontextmenu={(e) => onClick(e, item.stable_id)}
        onmouseenter={() => {
          /* invoke(SeelenCommand.SendSystemTrayIconAction, {
            id: item.stable_id,
            action: SystrayIconAction.HoverEnter,
          }); */
        }}
        onmousemove={() => {
          /* invoke(SeelenCommand.SendSystemTrayIconAction, {
            id: item.stable_id,
            action: SystrayIconAction.HoverMove,
          }); */
        }}
        onmouseleave={() => {
          /* invoke(SeelenCommand.SendSystemTrayIconAction, {
            id: item.stable_id,
            action: SystrayIconAction.HoverLeave,
          }); */
        }}
      >
        <div class="system-tray-item-icon-box">
          {#if !!item.icon_path}
            <img
              class="system-tray-item-icon"
              src={convertFileSrc(item.icon_path) + `?hash=${item.icon_image_hash || "null"}`}
              alt=""
            />
          {:else}
            <MissingIcon class="system-tray-item-icon" />
          {/if}
        </div>
        <span class="system-tray-item-label">
          {getItemName(item)}
        </span>
        <span
          class={["system-tray-item-pin", { "system-tray-item-pin-active": isPinned(item) }]}
          role="button"
          tabindex="0"
          title={isPinned(item) ? "Unpin" : "Pin"}
          onclick={(e) => onPinClick(e, item)}
          onkeydown={(e) => {
            if (e.key === "Enter" || e.key === " ") {
              onPinClick(e, item);
            }
          }}
        >
          <Icon iconName={isPinned(item) ? "TbPinnedFilled" : "TbPin"} />
        </span>
      </button>
    {/if}
  {/each}
</div>
