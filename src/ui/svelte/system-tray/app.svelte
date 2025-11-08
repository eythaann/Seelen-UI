<script lang="ts">
  import { SystrayIconAction, type SysTrayIconId } from "@seelen-ui/lib/types";
  import { state } from "./state.svelte";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { invoke, SeelenCommand } from "@seelen-ui/lib";
  import { MissingIcon } from "libs/ui/svelte/components/Icon";

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

  const GUIDS_TO_IGNORE = [
    "7820ae73-23e3-4229-82c1-e41cb67d5b9c", // speaker volument icon
    "7820ae74-23e3-4229-82c1-e41cb67d5b9c", // network icon
    "7820ae75-23e3-4229-82c1-e41cb67d5b9c", // battery icon
  ];
</script>

<div class="system-tray">
  {#each state.trayItems as item}
    {#if item.is_visible && (!item.guid || !GUIDS_TO_IGNORE.includes(item.guid))}
      <button
        class="system-tray-item"
        on:click={(e) => onClick(e, item.stable_id)}
        on:dblclick={(e) => onDoubleClick(e, item.stable_id)}
        on:contextmenu={(e) => onClick(e, item.stable_id)}
        on:mouseenter={() => {
          /* invoke(SeelenCommand.SendSystemTrayIconAction, {
            id: item.stable_id,
            action: SystrayIconAction.HoverEnter,
          }); */
        }}
        on:mousemove={() => {
          /* invoke(SeelenCommand.SendSystemTrayIconAction, {
            id: item.stable_id,
            action: SystrayIconAction.HoverMove,
          }); */
        }}
        on:mouseleave={() => {
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
          {item.tooltip || item.guid || `${item.window_handle?.toString(16)}::${item.uid}`}
        </span>
      </button>
    {/if}
  {/each}
</div>

<style>
  :global(body) {
    background-color: transparent;
    overflow: hidden;
  }

  :global(#root) {
    width: min-content;
    height: min-content;
  }
</style>
