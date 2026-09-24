<script lang="ts">
  import { SystrayIconAction, type SysTrayIcon } from "@seelen-ui/lib/types";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { invoke, SeelenCommand } from "@seelen-ui/lib";
  import { createSortable } from "@dnd-kit/svelte/sortable";
  import { MissingIcon } from "libs/ui/svelte/components/Icon";

  interface Props {
    item: SysTrayIcon;
    sortId: string;
    idx: number;
  }

  let { item, sortId, idx }: Props = $props();

  const sortable = createSortable({
    get id() {
      return sortId;
    },
    get index() {
      return idx;
    },
  });

  function onClick(event: MouseEvent) {
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
      id: item.stable_id,
      action,
    });
  }

  function onDoubleClick(e: MouseEvent) {
    e.preventDefault();
    e.stopPropagation();
    invoke(SeelenCommand.SendSystemTrayIconAction, {
      id: item.stable_id,
      action: SystrayIconAction.LeftDoubleClick,
    });
  }
</script>

<button
  {@attach sortable.attach}
  class="system-tray-item"
  class:is-dragging={sortable.isDragging}
  data-skin="transparent"
  onclick={onClick}
  ondblclick={onDoubleClick}
  oncontextmenu={onClick}
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
    {item.tooltip || item.guid || `${item.window_handle?.toString(16)}::${item.uid}`}
  </span>
</button>
