<script lang="ts">
  import { type FavPinnedItem, globalState } from "../state/mod.svelte";
  import { t } from "../i18n";
  import SortableAppItem from "./SortableAppItem.svelte";
  import FolderItem from "./FolderItem.svelte";
  import { DragDropProvider, DragOverlay } from "@dnd-kit/svelte";
  import { debounce } from "lodash";
  import { onDestroy, untrack } from "svelte";
  import { createDragDropManager } from "libs/ui/dnd";
  import { handleGridKeyDown, type SelectionScope } from "../keyboard-navigation";
  import { createItemIndex, getPinnedDragAction, type PinnedDragMove } from "../pinned-drag";
  import { getPinnedGrid, movePinnedItemToEmptySlot, reorderPinnedItems } from "../pinned-grid";

  type UniqueIdentifier = string | number;

  // Workaround for https://github.com/clauderic/dnd-kit/issues/2112
  const manager = createDragDropManager();
  onDestroy(() => manager.destroy());

  let selectedItemId = $state<string | null>(null);

  const scope: SelectionScope = {
    container: () => document,
    getSelected: () => selectedItemId,
    setSelected: (id) => {
      selectedItemId = id;
    },
  };

  let lastVersion: number | undefined;

  function isRenderable(item: FavPinnedItem): boolean {
    return item.type === "folder" || !!globalState.getMenuItem(item.itemId);
  }

  $effect(() => {
    const version = globalState.version;
    const pinnedItems = globalState.pinnedItems;
    const selected = untrack(() => selectedItemId);
    if (version === lastVersion && pinnedItems.some((item) => item.itemId === selected && isRenderable(item))) {
      return;
    }

    lastVersion = version;
    selectedItemId = pinnedItems.find(isRenderable)?.itemId ?? null;
  });

  // Reorder along the outer edges, leaving the middle available for folder creation.
  const POSITION_THRESHOLD = 0.2;
  let list: HTMLUListElement;
  let activeDropzoneId: string | null = $state(null);
  let pendingDropzoneId: string | null = null;
  let dragItems: FavPinnedItem[] | null = $state.raw(null);
  let dragBaseline: FavPinnedItem[] | null = null;
  let dropSlot: number | null = $state(null);
  let dragPreview: HTMLDivElement | null = $state.raw(null);
  let dragCapacity = $state(0);
  let dragIndex: ReadonlyMap<string, number> = new Map();
  let reorderFrame: number | null = null;

  const cells = $derived.by(() => {
    const grid = getPinnedGrid(dragItems ?? globalState.pinnedItems);
    while (grid.length < dragCapacity) grid.push(null);
    return grid;
  });

  // Debounced folder creation activation
  const FOLDER_CREATION_DELAY_MS = 200;
  const activateDropzone = debounce((targetId: string) => {
    pendingDropzoneId = null;
    activeDropzoneId = targetId;
  }, FOLDER_CREATION_DELAY_MS);

  function cancelDropzone() {
    activateDropzone.cancel();
    pendingDropzoneId = null;
    activeDropzoneId = null;
  }

  function cancelScheduledMove() {
    if (reorderFrame !== null) {
      cancelAnimationFrame(reorderFrame);
      reorderFrame = null;
    }
  }

  function resetDrag() {
    cancelScheduledMove();
    cancelDropzone();
    dragItems = null;
    dragBaseline = null;
    dropSlot = null;
    dragPreview = null;
    dragCapacity = 0;
    dragIndex = new Map();
  }

  function updateDragPreview(move: PinnedDragMove) {
    if (!dragItems) return;
    const action = getPinnedDragAction(move, dragIndex, POSITION_THRESHOLD);
    if (action.type === "reorder") {
      cancelDropzone();
      setDragItems(reorderPinnedItems(dragItems, action.sourceIndex, action.targetIndex));
    } else if (action.type === "dropzone") {
      if (pendingDropzoneId !== action.targetId && activeDropzoneId !== action.targetId) {
        activeDropzoneId = null;
        pendingDropzoneId = action.targetId;
        activateDropzone(action.targetId);
      }
    } else {
      cancelDropzone();
    }
  }

  function setDragItems(items: FavPinnedItem[]) {
    dragItems = items;
    dragIndex = createItemIndex(items.map((item) => item.itemId));
  }

  function captureDragPreview(source: Element | undefined): HTMLDivElement | null {
    if (!source?.childElementCount) return null;
    const preview = document.createElement("div");
    preview.className = source.classList.contains("folder") ? "folder" : "app";
    // Reuse the rendered icon and label without mounting another icon loader while dragging.
    preview.append(...Array.from(source.children, (child) => child.cloneNode(true)));
    return preview;
  }

  function attachSlotPreview(host: HTMLElement) {
    const preview = dragPreview;
    if (!preview) return;
    host.append(preview);
    return () => {
      if (preview.parentElement === host) preview.remove();
    };
  }

  function attachFloatingPreview(host: HTMLElement) {
    const preview = dragPreview?.cloneNode(true);
    if (!preview) return;
    host.append(preview);
    return () => host.replaceChildren();
  }

  function getVisibleGridCapacity(): number {
    const tile = list.querySelector<HTMLElement>(":scope > [data-item-id]");
    if (!tile?.offsetHeight) return 0;
    const gridStyle = getComputedStyle(list);
    const columns = gridStyle.gridTemplateColumns.split(/\s+/).filter((track) => track.endsWith("px")).length;
    const rowGap = Number.parseFloat(gridStyle.rowGap) || 0;
    const view = list.parentElement!;
    const viewStyle = getComputedStyle(view);
    const height = view.clientHeight - Number.parseFloat(viewStyle.paddingTop) -
      Number.parseFloat(viewStyle.paddingBottom);
    return columns * Math.ceil((height + rowGap) / (tile.offsetHeight + rowGap));
  }

  function queueDragMove() {
    if (reorderFrame !== null) return;

    reorderFrame = requestAnimationFrame(() => {
      reorderFrame = null;
      if (!dragItems || globalState.pinnedItems !== dragBaseline) {
        resetDrag();
        return;
      }

      // dnd-kit updates the position and collision after dispatching dragmove.
      const { source, target, position } = manager.dragOperation;
      const hit = getGridHit(position.current.x, position.current.y);
      dropSlot = hit?.kind === "slot" ? hit.slot : null;
      if (dropSlot !== null) {
        cancelDropzone();
        return;
      }
      const targetRect = target?.shape?.boundingRectangle;
      if (!source || !target || !targetRect) {
        cancelDropzone();
        return;
      }
      updateDragPreview({
        sourceId: String(source.id),
        targetId: String(target.id),
        sourceType: source.type?.toString(),
        targetType: target.type?.toString(),
        pointer: position.current,
        targetRect,
      });
    });
  }

  function getGridHit(x: number, y: number) {
    const cell = document.elementFromPoint(x, y)?.closest<HTMLElement>("[data-item-id], [data-empty-slot]");
    if (cell?.parentElement !== list) return;
    if (cell.dataset.emptySlot !== undefined) return { kind: "slot", slot: Number(cell.dataset.emptySlot) } as const;
    return { kind: "item", id: cell.dataset.itemId } as const;
  }

  function getDropTarget(event: Event | undefined, fallback: UniqueIdentifier | undefined) {
    // Read the release point even if it came in before the last scheduled drag move.
    return event instanceof PointerEvent
      ? getGridHit(event.clientX, event.clientY)
      : { kind: "item", id: fallback?.toString() } as const;
  }

  onDestroy(resetDrag);
</script>

<svelte:window onkeydown={(event) => handleGridKeyDown(event, scope)} />
<div
  class="pinned-view"
  class:has-slot-preview={dropSlot !== null && dragPreview !== null}
  class:has-drag-preview={dragPreview !== null}
>
  <DragDropProvider
    {manager}
    onDragStart={({ operation }) => {
      resetDrag();
      dragBaseline = globalState.pinnedItems;
      setDragItems(dragBaseline);
      dragCapacity = getVisibleGridCapacity();
      dragPreview = captureDragPreview(operation.source?.element);
    }}
    onDragMove={queueDragMove}
    onDragOver={(event) => {
      event.preventDefault();
    }}
    onDragEnd={(event) => {
      const { source, target } = event.operation;

      const items = dragItems;
      if (event.canceled || !items || !source || globalState.pinnedItems !== dragBaseline) {
        resetDrag();
        return;
      }
      cancelScheduledMove();

      const sourceId = String(source.id);
      const hit = getDropTarget(event.nativeEvent, target?.id);
      const sourceIndex = dragIndex.get(sourceId);
      if (hit?.kind === "slot" && sourceIndex !== undefined) {
        const updated = movePinnedItemToEmptySlot(items, sourceIndex, hit.slot);
        if (updated !== dragBaseline) globalState.pinnedItems = updated;
        resetDrag();
        return;
      }
      const targetId = hit?.kind === "item" ? hit.id : undefined;
      const targetIndex = targetId === undefined ? undefined : dragIndex.get(targetId);
      if (
        targetId !== undefined && sourceIndex !== undefined && targetIndex !== undefined && sourceIndex !== targetIndex
      ) {
        if (activeDropzoneId === targetId) {
          const sourceItem = items[sourceIndex]!;
          const targetItem = items[targetIndex]!;

          // Folder operations use the preview order to locate the result.
          if (items !== dragBaseline) globalState.pinnedItems = items;
          if (targetItem.type === "folder") {
            if (sourceItem.type === "folder") globalState.mergeFolders(sourceId, targetId);
            else globalState.addItemToFolder(targetId, sourceId);
          } else if (sourceItem.type === "app") {
            globalState.createFolder(sourceId, targetId, targetIndex);
          }
        } else {
          globalState.pinnedItems = reorderPinnedItems(items, sourceIndex, targetIndex);
        }
      } else if (items !== dragBaseline) {
        globalState.pinnedItems = items;
      }

      resetDrag();
    }}
  >
    <ul bind:this={list} class="pinned-view-list" role="listbox" aria-label={$t("applications")}>
      {#each cells as pinnedItem, idx (pinnedItem ? `item:${pinnedItem.itemId}` : `slot:${idx}`)}
        {#if pinnedItem === null}
          <li
            aria-hidden="true"
            class="pinned-view-slot"
            class:is-drop-target={dropSlot === idx}
            data-empty-slot={idx}
          >
            {#if dropSlot === idx && dragPreview}
              <div class="pinned-slot-preview" {@attach attachSlotPreview}></div>
            {/if}
          </li>
        {:else if pinnedItem.type === "app"}
          {@const item = globalState.getMenuItem(pinnedItem.itemId)}
          {#if item}
            <SortableAppItem
              {item}
              {idx}
              isActiveDropzone={activeDropzoneId === pinnedItem.itemId}
              {scope}
            />
          {/if}
        {:else if pinnedItem.type === "folder"}
          {@const folder = pinnedItem}
          <FolderItem
            {folder}
            {idx}
            isActiveDropzone={activeDropzoneId === pinnedItem.itemId}
            {scope}
          />
        {/if}
      {/each}
    </ul>
    <div class="pinned-drag-overlay" aria-hidden="true">
      <DragOverlay dropAnimation={null}>
        {#if dragPreview}
          <div {@attach attachFloatingPreview}></div>
        {/if}
      </DragOverlay>
    </div>
  </DragDropProvider>

  {#if globalState.pinnedItems.length === 0}
    <div class="pinned-view-empty">
      <p>{$t("welcome_message")}</p>
      <p>{$t("welcome_message2")}</p>
    </div>
  {/if}
</div>

<style>
  .pinned-view-slot {
    position: relative;
  }

  .pinned-slot-preview {
    position: absolute;
    inset: 0;
    pointer-events: none;
  }

  .pinned-drag-overlay {
    position: absolute;
    pointer-events: none;
  }

  .has-drag-preview .pinned-view-list > :global(.is-dragging),
  .has-slot-preview .pinned-drag-overlay :global([data-dnd-dragging]) {
    visibility: hidden;
  }
</style>
