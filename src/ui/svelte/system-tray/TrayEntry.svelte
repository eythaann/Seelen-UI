<script lang="ts">
  import { SystrayIconAction, type SysTrayIcon } from "@seelen-ui/lib/types";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { invoke, SeelenCommand } from "@seelen-ui/lib";
  import { createSortable } from "@dnd-kit/svelte/sortable";
  import { MissingIcon } from "libs/ui/svelte/components/Icon";
  import Icon from "libs/ui/svelte/components/Icon/Icon.svelte";

  interface Props {
    item: SysTrayIcon;
    index: number;
  }

  let { item, index }: Props = $props();

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

  function setPinned(id: string, promoted: boolean) {
    invoke(SeelenCommand.SetSystemTrayIconPromoted, {
      id,
      promoted,
    });
  }
</script>

<li
  {@attach sortable.attach}
  class="system-tray-entry"
  data-dragging={sortable.isDragging}
>
  <button
    class="system-tray-item"
    data-skin="transparent"
    onclick={(e) => onClick(e, item.registryKey)}
    ondblclick={(e) => onDoubleClick(e, item.registryKey)}
    onauxclick={(e) => onClick(e, item.registryKey)}
    oncontextmenu={(e) => onClick(e, item.registryKey)}
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
        <img
          class="system-tray-item-icon"
          src={convertFileSrc(item.iconPath) + `?hash=${item.iconImageHash || "null"}`}
          alt=""
        />
      {:else}
        <MissingIcon class="system-tray-item-icon" />
      {/if}
    </div>

    <span class="system-tray-item-label">
      {item.tooltip || item.guid || `${item.windowHandle?.toString(16)}::${item.uid}`}
    </span>
  </button>

  <button
    data-skin="transparent"
    class="system-tray-item-pin"
    onclick={() => setPinned(item.registryKey, !item.isPromoted)}
  >
    <Icon name={item.isPromoted ? "TbPinnedOff" : "TbPin"} />
  </button>
</li>
