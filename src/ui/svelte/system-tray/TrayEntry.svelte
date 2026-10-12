<script lang="ts">
  import { SystrayIconAction, type SysTrayIcon } from "@seelen-ui/lib/types";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { invoke, SeelenCommand, Widget } from "@seelen-ui/lib";
  import { Alignment, type ContextMenu } from "@seelen-ui/lib/types";
  import { createSortable } from "@dnd-kit/svelte/sortable";
  import { onDestroy, onMount } from "svelte";
  import { MissingIcon } from "libs/ui/svelte/components/Icon";
  import { t } from "./i18n";

  interface Props {
    item: SysTrayIcon;
    index: number;
  }

  let { item, index }: Props = $props();

  const itemLabel = $derived(
    item.tooltip || item.guid || `${item.windowHandle?.toString(16)}::${item.uid}`,
  );

  const sortable = createSortable({
    get id() {
      return item.registryKey;
    },
    get index() {
      return index;
    },
  });

  function onClick(event: MouseEvent, id: string) {
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

  function onDoubleClick(e: MouseEvent, id: string) {
    e.preventDefault();
    e.stopPropagation();
    invoke(SeelenCommand.SendSystemTrayIconAction, {
      id,
      action: SystrayIconAction.LeftDoubleClick,
    });
  }

  function onDragStart(event: DragEvent, id: string) {
    if (!event.dataTransfer) return;
    const payload = `seelen-system-tray:${id}`;
    event.dataTransfer.effectAllowed = "copy";
    event.dataTransfer.setData("application/x-seelen-system-tray-icon", payload);
    event.dataTransfer.setData("text/plain", payload);
  }

  const contextMenuEvent = `systemTrayContextMenu:${crypto.randomUUID()}`;
  let unlistenContextMenu: (() => void) | undefined;

  onMount(() => {
    Widget.self.webview.listen(contextMenuEvent, ({ payload }) => {
      const { key } = payload as { key: string };
      if (key === "pin") {
        invoke(SeelenCommand.SetSystemTrayIconPromoted, {
          id: item.registryKey,
          promoted: !item.isPromoted,
        });
      } else if (key === "application-menu") {
        invoke(SeelenCommand.SendSystemTrayIconAction, {
          id: item.registryKey,
          action: SystrayIconAction.RightClick,
        });
      }
    }).then((unlisten) => {
      unlistenContextMenu = unlisten;
    });
  });

  onDestroy(() => {
    unlistenContextMenu?.();
  });

  function showContextMenu(event: MouseEvent) {
    event.preventDefault();
    event.stopPropagation();
    const menu: ContextMenu = {
      identifier: crypto.randomUUID(),
      items: [
        {
          type: "Item",
          key: "pin",
          icon: item.isPromoted ? "TbPinnedOff" : "TbPin",
          label: item.isPromoted ? $t("unpin_from_toolbar") : $t("pin_to_toolbar"),
          callbackEvent: contextMenuEvent,
        },
        {
          type: "Item",
          key: "application-menu",
          label: $t("application_menu"),
          callbackEvent: contextMenuEvent,
        },
      ],
      alignX: Alignment.Center,
      alignY: Alignment.Start,
    };
    invoke(SeelenCommand.TriggerContextMenu, { menu, forwardTo: null });
  }
</script>

<li {@attach sortable.attach} class="system-tray-entry" data-dragging={sortable.isDragging}>
  <button
    class="system-tray-item"
    data-skin="transparent"
    aria-label={itemLabel}
    title={itemLabel}
    draggable="true"
    ondragstart={(event) => onDragStart(event, item.registryKey)}
    onclick={(e) => onClick(e, item.registryKey)}
    ondblclick={(e) => onDoubleClick(e, item.registryKey)}
    onauxclick={(e) => e.button === 1 && onClick(e, item.registryKey)}
    oncontextmenu={showContextMenu}
    onmouseenter={() => {
      /* invoke(SeelenCommand.SendSystemTrayIconAction, {
        id: item.registryKey,
        action: SystrayIconAction.HoverEnter,
      }); */
    }}
    onmousemove={() => {
      /* invoke(SeelenCommand.SendSystemTrayIconAction, {
        id: item.registryKey,
        action: SystrayIconAction.HoverMove,
      }); */
    }}
    onmouseleave={() => {
      /* invoke(SeelenCommand.SendSystemTrayIconAction, {
        id: item.registryKey,
        action: SystrayIconAction.HoverLeave,
      }); */
    }}
  >
    <div class="system-tray-item-icon-box">
      {#if !!item.iconPath}
        {#if item.isGlyph}
          <!-- single color glyph drawn for the taskbar theme, tinted by the theme instead -->
          <span
            class="system-tray-item-icon system-tray-item-icon-glyph"
            style:--icon-url={`url("${convertFileSrc(item.iconPath)}?hash=${item.iconImageHash || "null"}")`}
          ></span>
        {:else}
          <img
            class="system-tray-item-icon"
            src={convertFileSrc(item.iconPath) + `?hash=${item.iconImageHash || "null"}`}
            alt=""
          />
        {/if}
      {:else}
        <MissingIcon class="system-tray-item-icon" />
      {/if}
    </div>

  </button>
</li>
