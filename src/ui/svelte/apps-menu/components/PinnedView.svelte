<script lang="ts">
  import { globalState } from "../state/mod.svelte";
  import { t } from "../i18n";
  import AppItem from "./AppItem.svelte";
  import FolderItem from "./FolderItem.svelte";
  import { arrayMove } from "../utils";
  import { DragDropProvider } from "@dnd-kit-svelte/svelte";
  import { debounce } from "lodash";

  import type { StartMenuItem } from "@seelen-ui/lib/types";
  import type { FavFolderItem } from "../state/mod.svelte";

  type UniqueIdentifier = string | number;

  interface Props {
    onContextMenu: (event: MouseEvent, item: StartMenuItem | FavFolderItem) => void;
  }

  let { onContextMenu }: Props = $props();

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

<div class="pinned-view">
  <div class="pinned-view-list">
    <DragDropProvider
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
          globalState.pinnedItems = arrayMove(globalState.pinnedItems, sourceIndex, targetIndex);
        }

        if (source.type === "folder" && target.type === "app") {
          return;
        }
        activateDropzone(target.id);
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
            <AppItem
              {item}
              {idx}
              {onContextMenu}
              isActiveDropzone={activeDropzoneId === pinnedItem.itemId}
            />
          {/if}
        {:else if pinnedItem.type === "folder"}
          {@const folder = pinnedItem}
          <FolderItem
            {folder}
            {idx}
            {onContextMenu}
            isActiveDropzone={activeDropzoneId === pinnedItem.itemId}
          />
        {/if}
      {/each}
    </DragDropProvider>
  </div>

  {#if globalState.pinnedItems.length === 0}
    <div class="pinned-view-empty">
      <p>{$t("welcome_message")}</p>
      <p>{$t("welcome_message2")}</p>
    </div>
  {/if}
</div>
