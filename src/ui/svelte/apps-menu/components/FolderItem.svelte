<script lang="ts">
  import type { FavFolderItem } from "../state/mod.svelte";
  import { FileIcon } from "libs/ui/svelte/components/Icon";
  import { globalState } from "../state/mod.svelte";
  import { t } from "../i18n";
  import { createDraggable, createDroppable } from "@dnd-kit/svelte";
  import FolderModal from "./FolderModal.svelte";
  import type { StartMenuItem } from "@seelen-ui/lib/types";

  interface Props {
    folder: FavFolderItem;
    idx: number;
    onContextMenu: (event: MouseEvent, folder: FavFolderItem | StartMenuItem) => void;
    isActiveDropzone?: boolean;
  }

  let { folder, idx, onContextMenu, isActiveDropzone = false }: Props = $props();

  let isModalOpen = $state(false);
  $effect(() => {
    if (!globalState.showing) {
      isModalOpen = false;
    }
  });

  const isPreselected = $derived(
    globalState.preselectedItem === folder.itemId || (idx === 0 && !globalState.preselectedItem),
  );

  const folderName = $derived(folder.name || $t("folder"));
  const expandedItems = $derived(
    folder.itemIds.map((id) => ({
      id,
      item: globalState.getMenuItem(id)!,
    })),
  );

  const draggable = $derived(
    createDraggable({
      id: folder.itemId,
      type: "folder",
    }),
  );

  const droppable = $derived(
    createDroppable({
      id: folder.itemId,
      type: "folder",
    }),
  );

  function openModal() {
    isModalOpen = true;
  }

  function closeModal() {
    isModalOpen = false;
  }
</script>

<button
  {@attach draggable.attach}
  {@attach droppable.attach}
  data-item-id={folder.itemId}
  class="folder"
  class:preselected={isPreselected && globalState.searchQuery}
  class:is-dragging={draggable.isDragging}
  class:is-dropping={draggable.isDropping}
  class:is-drop-target={isActiveDropzone}
  onclick={openModal}
  oncontextmenu={(event) => {
    onContextMenu(event, folder);
  }}
  onfocus={() => {
    globalState.preselectedItem = folder.itemId;
  }}
>
  <div class="folder-grid">
    {#each expandedItems.slice(0, 4) as app}
      <div class="folder-preview">
        <FileIcon class="folder-preview-icon" path={app.item.path} umid={app.item.umid} />
      </div>
    {/each}
  </div>
  <div class="folder-name" title={folderName}>
    {folderName}
  </div>
</button>

{#if isModalOpen}
  <FolderModal {folder} onClose={closeModal} {onContextMenu} />
{/if}
