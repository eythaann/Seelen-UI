<script lang="ts">
  import type { StartMenuItem } from "@seelen-ui/lib/types";
  import { FileIcon } from "libs/ui/svelte/components/Icon";
  import { globalState } from "../state.svelte";
  import { t } from "../i18n";

  interface Props {
    item: StartMenuItem;
    pinned?: boolean;
    class?: string;
  }

  let { item, pinned = false, class: className = "" }: Props = $props();

  let contextMenuVisible = $state(false);
  let contextMenuX = $state(0);
  let contextMenuY = $state(0);

  function handleContextMenu(event: MouseEvent) {
    event.preventDefault();
    contextMenuX = event.clientX;
    contextMenuY = event.clientY;
    contextMenuVisible = true;
  }

  function handleTogglePin() {
    globalState.togglePin(item);
    contextMenuVisible = false;
  }

  function handleClick() {
    // TODO: Launch application
  }

  function handleClickOutside(event: MouseEvent) {
    const target = event.target as HTMLElement;
    if (!target.closest(".context-menu")) {
      contextMenuVisible = false;
    }
  }

  $effect(() => {
    if (contextMenuVisible) {
      document.addEventListener("click", handleClickOutside);
      // document.addEventListener("contextmenu", handleClickOutside);

      return () => {
        document.removeEventListener("click", handleClickOutside);
        // document.removeEventListener("contextmenu", handleClickOutside);
      };
    }
    return undefined;
  });
</script>

<div
  class="app-item"
  onclick={handleClick}
  oncontextmenu={handleContextMenu}
  role="button"
  tabindex="0"
  onkeydown={(e) => {
    if (e.key === "Enter" || e.key === " ") {
      e.currentTarget.click();
    }
  }}
>
  <FileIcon class="app-item-icon" path={item.path} umid={item.umid} />
  <div class="app-item-name">
    {item.display_name}
  </div>
</div>

{#if contextMenuVisible}
  <div
    class="context-menu"
    style="left: {contextMenuX}px; top: {contextMenuY}px;"
    onclick={(e) => e.stopPropagation()}
    oncontextmenu={(e) => e.stopPropagation()}
    role="menu"
    tabindex="0"
    onkeydown={(e) => {
      if (e.key === "Enter" || e.key === " ") {
        e.currentTarget.click();
      }
    }}
  >
    <button class="context-menu-item" onclick={handleTogglePin}>
      {pinned || globalState.isPinned(item) ? $t("unpin") : $t("pin")}
    </button>
  </div>
{/if}

<style>
  :global(.context-menu) {
    position: fixed;
    z-index: 1000;
  }
</style>
