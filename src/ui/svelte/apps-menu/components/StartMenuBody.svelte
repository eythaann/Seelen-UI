<script lang="ts">
  import { globalState } from "../state.svelte";
  import { StartView } from "../constants";
  import PinnedView from "./PinnedView.svelte";
  import AllAppsView from "./AllAppsView.svelte";
  import { t } from "../i18n";
  import { Icon } from "libs/ui/svelte/components/Icon";
  import { invoke, SeelenCommand } from "@seelen-ui/lib";

  let contextMenu = $state<HTMLDivElement>();
  let contextMenuX = $state(0);
  let contextMenuY = $state(0);
  let activeContextMenuItem = $state<string | null>(null);

  const activeContextMenuItemData = $derived.by(() => {
    if (!activeContextMenuItem) return null;
    const item = globalState.allItems.find(
      (item) => (item.umid || item.path) === activeContextMenuItem,
    );
    return item || null;
  });

  const activeContextMenuItemPinned = $derived(
    activeContextMenuItemData ? globalState.isPinned(activeContextMenuItemData) : false,
  );

  const activeContextMenuIsFolder = $derived.by(() => {
    if (!activeContextMenuItem) return false;
    const folder = globalState.pinnedItems.find(
      (item) => item.type === "folder" && item.itemId === activeContextMenuItem,
    );
    return !!folder;
  });

  function handleContextMenu(event: MouseEvent, itemId: string) {
    contextMenuX = event.clientX;
    contextMenuY = event.clientY;
    activeContextMenuItem = itemId;
  }

  function handleTogglePin() {
    if (activeContextMenuItemData) {
      globalState.togglePin(activeContextMenuItemData);
    }
    activeContextMenuItem = null;
  }

  function handleDisbandFolder() {
    if (activeContextMenuItem) {
      globalState.disbandFolder(activeContextMenuItem);
    }
    activeContextMenuItem = null;
  }

  $effect(() => {
    if (activeContextMenuItem === null) return;

    const handleOutside = (event: MouseEvent) => {
      if (!contextMenu?.contains(event.target as HTMLElement)) {
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

{#if activeContextMenuItem}
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
    {#if activeContextMenuIsFolder}
      <button class="context-menu-item" onclick={handleDisbandFolder}>
        <Icon iconName="GiExpand" />
        <span>{$t("disband")}</span>
      </button>
    {:else}
      <button class="context-menu-item" onclick={handleTogglePin}>
        <Icon iconName={activeContextMenuItemPinned ? "TbPinnedOff" : "TbPin"} />
        <span>{activeContextMenuItemPinned ? $t("unpin") : $t("pin")}</span>
      </button>

      {#if activeContextMenuItemData?.path}
        <button
          class="context-menu-item"
          onclick={() => {
            activeContextMenuItem = null;
            globalState.showing = false;

            invoke(SeelenCommand.SelectFileOnExplorer, { path: activeContextMenuItemData.path });
          }}
        >
          <Icon iconName="MdOutlineMyLocation" />
          <span>{$t("open_file_location")}</span>
        </button>
      {/if}

      {#if activeContextMenuItemData?.umid || activeContextMenuItemData?.path
          .toLowerCase()
          .endsWith(".lnk")}
        <button
          class="context-menu-item"
          onclick={() => {
            activeContextMenuItem = null;
            globalState.showing = false;

            let program = activeContextMenuItemData.umid
              ? `shell:AppsFolder\\${activeContextMenuItemData.umid}`
              : activeContextMenuItemData.path;
            invoke(SeelenCommand.Run, {
              program,
              args: null,
              workingDir: null,
              elevated: true,
            });
          }}
        >
          <Icon iconName="MdOutlineAdminPanelSettings" />
          <span>{$t("run_as_admin")}</span>
        </button>
      {/if}
    {/if}
  </div>
{/if}

<style>
  :global(.context-menu) {
    position: fixed;
    z-index: 1000;
  }
</style>
