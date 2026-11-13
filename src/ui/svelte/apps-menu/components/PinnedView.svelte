<script lang="ts">
  import { globalState, type FavPinnedItem } from "../state/mod.svelte";
  import { t } from "../i18n";
  import SortableAppItem from "./SortableAppItem.svelte";
  import FolderItem from "./FolderItem.svelte";
  import { arrayMove } from "../utils";
  import { DragDropProvider } from "@dnd-kit/svelte";
  import { debounce } from "lodash";
  import { onDestroy, untrack } from "svelte";
  import { createDragDropManager } from "libs/ui/dnd";
  import { handleGridKeyDown, type SelectionScope } from "../keyboard-navigation";

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

  // Position-based sorting threshold (20% of item width)
  const POSITION_THRESHOLD = 0.2;
  let activeDropzoneId: UniqueIdentifier | null = $state(null);

  // Debounced folder creation activation
  const FOLDER_CREATION_DELAY_MS = 200;
  const activateDropzone = debounce((targetId: UniqueIdentifier) => {
    activeDropzoneId = targetId;
  }, FOLDER_CREATION_DELAY_MS);

  function cancelDropzone() {
    activateDropzone.cancel();
    activeDropzoneId = null;
  }
</script>

<svelte:window onkeydown={(event) => handleGridKeyDown(event, scope)} />
<div class="pinned-view">
  <ul class="pinned-view-list" role="listbox" aria-label={$t("applications")}>
    <DragDropProvider
      {manager}
      onDragMove={(event) => {
        const { source, target } = event.operation;

        // No collision - cancel dropzone
        if (!source || !target || source.id === target.id) {
          cancelDropzone();
          return;
        }

        const sourceIndex = globalState.pinnedItems.findIndex((item) => item.itemId === source.id);
        const targetIndex = globalState.pinnedItems.findIndex((item) => item.itemId === target.id);

        if (sourceIndex === -1 || targetIndex === -1) {
          return;
        }

        const sourceElement = source.element;
        const targetElement = target.element;
        if (!sourceElement || !targetElement) {
          return;
        }

        const targetRect = targetElement.getBoundingClientRect();
        const sourceRect = sourceElement.getBoundingClientRect();

        // Calculate relative position (0 = left edge, 1 = right edge)
        const sourceCenterX = sourceRect.left + sourceRect.width / 2;
        const targetLeft = targetRect.left;
        const targetWidth = targetRect.width;
        const relativePosition = (sourceCenterX - targetLeft) / targetWidth;

        let shouldSort = false;
        if (sourceIndex > targetIndex) {
          shouldSort = relativePosition < POSITION_THRESHOLD;
        } else if (sourceIndex < targetIndex) {
          shouldSort = relativePosition > 1 - POSITION_THRESHOLD;
        }

        if (shouldSort && sourceIndex !== targetIndex) {
          cancelDropzone();
          globalState.pinnedItems = arrayMove(globalState.pinnedItems, sourceIndex, targetIndex);
          return;
        }

        if (source.type === "folder" && target.type === "app") {
          return;
        }
        activateDropzone(target.id);
      }}
      onDragOver={(event) => {
        event.preventDefault();
      }}
      onDragEnd={(event) => {
        const { source, target } = event.operation;
        activateDropzone.cancel();

        // Create folder if dropzone was active
        if (activeDropzoneId && source && target) {
          const sourceId = source.id.toString();
          const targetId = activeDropzoneId.toString();

          // Find source and target items
          const sourceItem = globalState.pinnedItems.find((item) => item.itemId === sourceId);
          const targetIndex = globalState.pinnedItems.findIndex((item) => item.itemId === targetId);
          const targetItem = globalState.pinnedItems[targetIndex];

          if (!targetItem || !sourceItem) {
            cancelDropzone();
            return;
          }

          // Case 1: Source is folder + Target is folder - merge folders
          if (sourceItem.type === "folder" && targetItem.type === "folder") {
            globalState.mergeFolders(sourceItem.itemId, targetItem.itemId);
          }
          // Case 2: Source is app + Target is folder - add app to existing folder
          else if (sourceItem.type === "app" && targetItem.type === "folder") {
            globalState.addItemToFolder(targetItem.itemId, sourceId);
          }
          // Case 3: Source is app + Target is app - create new folder with both items
          else if (sourceItem.type === "app" && targetItem.type === "app") {
            // Don't create folder if dragging onto self
            if (sourceId !== targetId) {
              globalState.createFolder(sourceId, targetId, targetIndex);
            }
          }
        }

        cancelDropzone();
      }}
    >
      {#each globalState.pinnedItems as pinnedItem, idx (pinnedItem.itemId)}
        {#if pinnedItem.type === "app"}
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
    </DragDropProvider>
  </ul>

  {#if globalState.pinnedItems.length === 0}
    <div class="pinned-view-empty">
      <p>{$t("welcome_message")}</p>
      <p>{$t("welcome_message2")}</p>
    </div>
  {/if}
</div>
