<script lang="ts">
  import type { StartMenuItem } from "@seelen-ui/lib/types";
  import { globalState } from "../state.svelte";
  import { StartView } from "../constants";
  import PinnedView from "./PinnedView.svelte";
  import AllAppsView from "./AllAppsView.svelte";
  import { t } from "../i18n";

  let contextMenu = $state<HTMLDivElement>();
  let contextMenuVisible = $state(false);
  let contextMenuX = $state(0);
  let contextMenuY = $state(0);
  let activeContextMenuItem = $state<string | null>(null);

  const activeContextMenuItemData = $derived.by(() => {
    if (!activeContextMenuItem) return null;
    const item = globalState.allItems.find(
      (item) => (item.umid || item.path) === activeContextMenuItem
    );
    return item || null;
  });

  const activeContextMenuItemPinned = $derived(
    activeContextMenuItemData ? globalState.isPinned(activeContextMenuItemData) : false
  );

  function handleContextMenu(event: MouseEvent, itemId: string) {
    contextMenuX = event.clientX;
    contextMenuY = event.clientY;
    contextMenuVisible = true;
    activeContextMenuItem = itemId;
  }

  function handleTogglePin() {
    if (activeContextMenuItemData) {
      globalState.togglePin(activeContextMenuItemData);
    }
    contextMenuVisible = false;
    activeContextMenuItem = null;
  }

  $effect(() => {
    if (!contextMenuVisible) return;

    const handleOutside = (event: MouseEvent) => {
      if (!contextMenu?.contains(event.target as HTMLElement)) {
        contextMenuVisible = false;
        activeContextMenuItem = null;
      }
    };

    let timeoutId = setTimeout(() => {
      document.addEventListener("click", handleOutside);
      document.addEventListener("contextmenu", handleOutside);
    }, 0);

    return () => {
      clearTimeout(timeoutId);
      document.removeEventListener("click", handleOutside);
      document.removeEventListener("contextmenu", handleOutside);
    };
  });
</script>

<div class="apps-menu-body">
  {#if globalState.view === StartView.Favorites}
    <PinnedView onContextMenu={handleContextMenu} />
  {:else if globalState.view === StartView.All}
    <AllAppsView onContextMenu={handleContextMenu} />
  {/if}
</div>

{#if contextMenuVisible}
  <div
    bind:this={contextMenu}
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
      {activeContextMenuItemPinned ? $t("unpin") : $t("pin")}
    </button>
  </div>
{/if}

<style>
  :global(.context-menu) {
    position: fixed;
    z-index: 1000;
  }
</style>
