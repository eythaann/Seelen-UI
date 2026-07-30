<script lang="ts">
  import type { FavFolderItem } from "../state/mod.svelte";
  import { globalState } from "../state/mod.svelte";
  import { t } from "../i18n";
  import { DragDropProvider } from "@dnd-kit/svelte";
  import { onDestroy } from "svelte";
  import SortableAppItem from "./SortableAppItem.svelte";
  import { arrayMove } from "../utils";
  import { createDragDropManager } from "libs/ui/dnd";
  import {
    navigateInDirection,
    selectPreselectedOrFirst,
    type SelectionScope,
  } from "../keyboard-navigation";

  interface Props {
    folder: FavFolderItem;
    onClose: () => void;
  }

  let { folder, onClose }: Props = $props();

  // Workaround for https://github.com/clauderic/dnd-kit/issues/2112
  const manager = createDragDropManager();
  onDestroy(() => manager.destroy());

  const folderName = $derived(folder.name || $t("folder"));
  const expandedItems = $derived(
    folder.itemIds.map((id) => ({
      id,
      item: globalState.getMenuItem(id)!,
    })),
  );

  let dialog: HTMLDialogElement;
  // svelte-ignore state_referenced_locally
  let selectedItemId = $state<string | null>(expandedItems[0]?.id ?? null);

  const scope: SelectionScope = {
    container: () => dialog ?? document,
    getSelected: () => selectedItemId,
    setSelected: (id) => {
      selectedItemId = id;
    },
  };

  // `open` alone doesn't move focus into the dialog; showModal() does
  // (and is what actually makes it modal / focus-trapped). showModal()
  // would otherwise auto-focus the name input (first focusable descendant),
  // so focus the dialog itself instead — the name field is opt-in via click.
  $effect(() => {
    if (!dialog) return;
    dialog.showModal();
    dialog.focus();
  });

  function handleNameInputKeyDown(event: KeyboardEvent) {
    event.stopPropagation();
    if (event.key === "Enter") {
      event.preventDefault();
      (event.currentTarget as HTMLInputElement).blur();
    }
  }

  function handleDialogKeyDown(event: KeyboardEvent) {
    // Contain all keyboard navigation inside the modal.
    event.stopPropagation();

    switch (event.key) {
      case "Enter":
        event.preventDefault();
        selectPreselectedOrFirst(scope)?.click();
        break;
      case "ArrowUp":
        event.preventDefault();
        navigateInDirection("up", scope);
        break;
      case "ArrowDown":
        event.preventDefault();
        navigateInDirection("down", scope);
        break;
      case "ArrowLeft":
        event.preventDefault();
        navigateInDirection("left", scope);
        break;
      case "ArrowRight":
        event.preventDefault();
        navigateInDirection("right", scope);
        break;
    }
  }
</script>

<dialog
  bind:this={dialog}
  class="folder-modal"
  closedby="any"
  onkeydown={handleDialogKeyDown}
  onclose={onClose}
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
        onkeydown={handleNameInputKeyDown}
      />

      <DragDropProvider
        {manager}
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
              (item) => !(item.type === "folder" && item.itemId === folder.itemId),
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
        <ul role="listbox" class="folder-modal-items">
          {#each expandedItems as { id, item }, idx (id)}
            <SortableAppItem {item} {idx} isInsideFolder={true} {scope} />
          {/each}
        </ul>
      </DragDropProvider>
    </div>
  </div>
</dialog>
