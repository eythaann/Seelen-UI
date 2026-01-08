<script lang="ts">
  import type { FavFolderItem } from "../state.svelte";
  import { globalState } from "../state.svelte";
  import { t } from "../i18n";
  import { DragDropProvider } from "@dnd-kit-svelte/svelte";
  import AppItem from "./AppItem.svelte";
  import { arrayMove } from "../utils";

  interface Props {
    folder: FavFolderItem;
    onClose: () => void;
    onContextMenu: (event: MouseEvent, itemId: string) => void;
  }

  let { folder, onClose, onContextMenu }: Props = $props();

  const folderName = $derived(folder.name || $t("folder"));
  const expandedItems = $derived(
    folder.itemIds.map((id) => ({
      id,
      item: globalState.getMenuItem(id)!,
    }))
  );

  let dialog = $state<HTMLDialogElement>();
</script>

<dialog
  bind:this={dialog}
  open
  class="folder-modal"
  closedby="any"
  onkeydown={(e) => {
    e.stopPropagation(); // avoid window keydown events for navigation
  }}
  onclose={onClose}
>
  <DragDropProvider
    onDragOver={(event) => {
      const { source, target } = event.operation;
      if (!source || !target || source.id === target.id) {
        return;
      }

      const oldIndex = expandedItems.findIndex((item) => item.id === source.id);
      const newIndex = expandedItems.findIndex((item) => item.id === target.id);

      if (oldIndex !== -1 && newIndex !== -1) {
        let newItems = arrayMove(expandedItems, oldIndex, newIndex);
        globalState.updateFolder(folder.itemId, {
          itemIds: newItems.map((item) => item.id),
        });
      }
    }}
    onDragEnd={(event) => {
      const { source } = event.operation;
      if (!dialog || !source || !source.element) {
        return;
      }

      let rect = source.element.getBoundingClientRect();
      let dialogRect = dialog.getBoundingClientRect();

      const hasIntersection =
        rect.right > dialogRect.left &&
        rect.left < dialogRect.right &&
        rect.bottom > dialogRect.top &&
        rect.top < dialogRect.bottom;

      if (hasIntersection) {
        return;
      }

      const draggedItemId = source.id as string;
      const newItemIds = folder.itemIds.filter((id) => id !== draggedItemId);

      // Update folder with new items or handle folder removal
      if (newItemIds.length >= 2) {
        globalState.updateFolder(folder.itemId, { itemIds: newItemIds });
      }
      // Folder has only 1 item left, convert to app and remove folder
      else if (newItemIds.length === 1) {
        const remainingItemId = newItemIds[0]!;
        globalState.pinnedItems = globalState.pinnedItems
          .filter((item) => !(item.type === "folder" && item.itemId === folder.itemId))
          .concat({ type: "app", itemId: remainingItemId });
      }
      // Folder is empty, just remove it
      else {
        globalState.pinnedItems = globalState.pinnedItems.filter(
          (item) => !(item.type === "folder" && item.itemId === folder.itemId)
        );
      }

      // Add dragged item as standalone app
      globalState.pinnedItems = [
        ...globalState.pinnedItems,
        { type: "app", itemId: draggedItemId },
      ];
      onClose();
    }}
  >
    <div class="folder-modal">
      <div class="folder-modal-content">
        <input
          type="text"
          data-skin="transparent"
          class="folder-modal-name"
          value={folderName}
          placeholder={$t("folder")}
          oninput={(e) => {
            globalState.updateFolder(folder.itemId, { name: e.currentTarget.value });
          }}
        />

        <div class="folder-modal-items">
          {#each expandedItems as { id, item }, idx (id)}
            <AppItem
              {item}
              {idx}
              isInsideFolder={true}
              onContextMenu={(event) => {
                onContextMenu(event, id);
              }}
            />
          {/each}
        </div>
      </div>
    </div>
  </DragDropProvider>
</dialog>
