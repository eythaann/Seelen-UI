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

  function handleKeyDown(e: KeyboardEvent) {
    e.stopPropagation();
    if (e.key === "Escape") {
      onClose();
    }
  }

  function handleOverlayClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      onClose();
    }
  }
</script>

<div
  role="dialog"
  tabindex="-1"
  class="folder-modal-overlay"
  onclick={handleOverlayClick}
  onkeydown={handleKeyDown}
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
              onContextMenu={(event) => {
                onContextMenu(event, id);
              }}
            />
          {/each}
        </div>
      </div>
    </div>
  </DragDropProvider>
</div>
