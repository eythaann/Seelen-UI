<script lang="ts">
  import type { StartMenuItem } from "@seelen-ui/lib/types";
  import type { FavFolderItem } from "../state/mod.svelte";
  import { globalState } from "../state/mod.svelte";
  import { StartView } from "../constants";
  import PinnedView from "./PinnedView.svelte";
  import AllAppsView from "./AllAppsView.svelte";
  import { t } from "../i18n";
  import { Icon } from "libs/ui/svelte/components/Icon";
  import { invoke, SeelenCommand } from "@seelen-ui/lib";

  let contextMenu = $state<HTMLDivElement>();
  let contextMenuX = $state(0);
  let contextMenuY = $state(0);
  let activeContextMenuItem = $state<StartMenuItem | FavFolderItem | null>(null);

  const activeContextMenuItemPinned = $derived(
    activeContextMenuItem && "path" in activeContextMenuItem
      ? globalState.isPinned(activeContextMenuItem)
      : false,
  );

  const activeContextMenuIsFolder = $derived(
    activeContextMenuItem ? "itemIds" in activeContextMenuItem : false,
  );

  function handleContextMenu(event: MouseEvent, item: StartMenuItem | FavFolderItem) {
    contextMenuX = event.clientX;
    contextMenuY = event.clientY;
    activeContextMenuItem = item;
  }

  function handleTogglePin() {
    if (activeContextMenuItem && "path" in activeContextMenuItem) {
      globalState.togglePin(activeContextMenuItem);
    }
    activeContextMenuItem = null;
  }

  function handleDisbandFolder() {
    if (activeContextMenuItem && "itemIds" in activeContextMenuItem) {
      globalState.disbandFolder(activeContextMenuItem.itemId);
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
    {:else if activeContextMenuItem && "path" in activeContextMenuItem}
      <button class="context-menu-item" onclick={handleTogglePin}>
        <Icon iconName={activeContextMenuItemPinned ? "TbPinnedOff" : "TbPin"} />
        <span>{activeContextMenuItemPinned ? $t("unpin") : $t("pin")}</span>
      </button>

      {#if activeContextMenuItem.path}
        <button
          class="context-menu-item"
          onclick={() => {
            if (activeContextMenuItem && "path" in activeContextMenuItem) {
              globalState.showing = false;
              invoke(SeelenCommand.SelectFileOnExplorer, { path: activeContextMenuItem.path });
              activeContextMenuItem = null;
            }
          }}
        >
          <Icon iconName="MdOutlineMyLocation" />
          <span>{$t("open_file_location")}</span>
        </button>
      {/if}

      {#if activeContextMenuItem.umid || activeContextMenuItem.path.toLowerCase().endsWith(".lnk")}
        <button
          class="context-menu-item"
          onclick={() => {
            if (activeContextMenuItem && "path" in activeContextMenuItem) {
              globalState.showing = false;
              let program = activeContextMenuItem.umid
                ? `shell:AppsFolder\\${activeContextMenuItem.umid}`
                : activeContextMenuItem.path;
              invoke(SeelenCommand.Run, {
                program,
                args: null,
                workingDir: null,
                elevated: true,
              });
              activeContextMenuItem = null;
            }
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
