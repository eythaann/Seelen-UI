<script lang="ts">
  import { invoke, SeelenCommand } from "@seelen-ui/lib";
  import { SeelenWegMode, WegItemType, WegPinnedItemsVisibility, WegTemporalItemsVisibility } from "@seelen-ui/lib/types";
  import { DragDropProvider, DragOverlay } from "@dnd-kit/svelte";
  import { move } from "@dnd-kit/helpers";
  import { BackgroundByLayers } from "libs/ui/svelte/components/BackgroundByLayers";
  import { t } from "../i18n/index.ts";
  import { dockState } from "../state/items.svelte.ts";
  import { settingsState, getDockContextMenuAlignment } from "../state/settings.svelte.ts";
  import { systemState } from "../state/system.svelte.ts";
  import { interactables, getWindowsForItem } from "../state/windows.svelte.ts";
  import { dockShouldBeHidden } from "../state/hidden.svelte.ts";
  import { getSeelenWegMenu } from "../dockMenu.ts";
  import { DND_PLUGINS, DND_SENSORS } from "libs/ui/dnd.ts";
  import type { SwItem } from "../types.ts";
  import DraggableItem from "./DraggableItem.svelte";
  import Separator from "./items/Separator.svelte";
  import StartMenu from "./items/StartMenu.svelte";
  import ShowDesktop from "./items/ShowDesktop.svelte";
  import RecycleBin from "./items/RecycleBin.svelte";
  import MediaSession from "./items/MediaSession.svelte";
  import UserApplication from "./items/UserApplication.svelte";

  const settings = $derived(settingsState.value as any);
  const isHorizontal = $derived(
    settings?.position === "Top" || settings?.position === "Bottom",
  );

  const visibleItems = $derived.by(() => {
    const pinnedVisibility = settings?.pinnedItemsVisibility as WegPinnedItemsVisibility;
    const temporalVisibility = settings?.temporalItemsVisibility as WegTemporalItemsVisibility;
    const monitor = systemState.currentMonitor;

    const showPinned =
      pinnedVisibility === WegPinnedItemsVisibility.Always || monitor.isPrimary;
    const filterByMonitor =
      temporalVisibility === WegTemporalItemsVisibility.OnMonitor;

    const windows = filterByMonitor
      ? interactables.value.filter((w) => w.monitor === monitor.id)
      : interactables.value;

    return dockState.items.filter((item) => {
      if (item.type !== "AppOrFile") {
        return showPinned;
      }
      if (item.pinned && showPinned) {
        return true;
      }
      return getWindowsForItem(item as any, windows).length > 0;
    });
  });

  const isEmpty = $derived(
    visibleItems.filter((c) => c.type !== WegItemType.Separator).length === 0,
  );

  function onContextMenu() {
    const { alignX, alignY } = getDockContextMenuAlignment(settingsState.position);
    invoke(SeelenCommand.TriggerContextMenu, {
      menu: { ...getSeelenWegMenu($t), alignX, alignY },
      forwardTo: null,
    });
  }

  function handleDragOver(event: any) {
    const newItems = move(dockState.items, event);
    dockState.items = newItems;
  }
</script>

<div
  role="toolbar"
  tabindex="0"
  data-has-margin={!!settings?.margin}
  data-size={settings?.mode === SeelenWegMode.FullWidth ? "full-width" : "min-content"}
  class="taskbar {settingsState.position.toLowerCase()}"
  class:horizontal={isHorizontal}
  class:vertical={!isHorizontal}
  class:hidden={dockShouldBeHidden.value}
  oncontextmenu={onContextMenu}
>
  <BackgroundByLayers id="weg-background" class="" />
  <div class="weg-items-container">
    <DragDropProvider plugins={DND_PLUGINS} sensors={DND_SENSORS} onDragOver={handleDragOver}>
      <div class="weg-items">
        {#if isEmpty}
          <span class="weg-empty-state-label">{$t("weg.empty")}</span>
        {:else}
          {#each visibleItems as item, index (item.id)}
            <DraggableItem {item} {index}>
              {#if item.type === WegItemType.AppOrFile}
                <UserApplication {item} />
              {:else if item.type === WegItemType.StartMenu}
                <StartMenu {item} />
              {:else if item.type === WegItemType.ShowDesktop}
                <ShowDesktop {item} />
              {:else if item.type === WegItemType.Media}
                <MediaSession {item} />
              {:else if item.type === WegItemType.Separator}
                <Separator {item} />
              {:else if item.type === WegItemType.TrashBin}
                <RecycleBin {item} />
              {/if}
            </DraggableItem>
          {/each}
        {/if}
      </div>

      <DragOverlay>
        {#snippet children(source)}
          {@const overlayItem = visibleItems.find((c) => c.id === source.id)}
          {#if overlayItem}
            {#if overlayItem.type === WegItemType.AppOrFile}
              <UserApplication item={overlayItem} isOverlay={true} />
            {:else if overlayItem.type === WegItemType.StartMenu}
              <StartMenu item={overlayItem} />
            {:else if overlayItem.type === WegItemType.ShowDesktop}
              <ShowDesktop item={overlayItem} />
            {:else if overlayItem.type === WegItemType.Media}
              <MediaSession item={overlayItem} />
            {:else if overlayItem.type === WegItemType.Separator}
              <Separator item={overlayItem} />
            {:else if overlayItem.type === WegItemType.TrashBin}
              <RecycleBin item={overlayItem} />
            {/if}
          {/if}
        {/snippet}
      </DragOverlay>
    </DragDropProvider>
  </div>
</div>
