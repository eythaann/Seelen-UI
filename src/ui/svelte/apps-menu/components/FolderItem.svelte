<script lang="ts">
  import type { FavFolderItem } from "../state.svelte";
  import { FileIcon } from "libs/ui/svelte/components/Icon";
  import { globalState } from "../state.svelte";
  import { t } from "../i18n";
  import { useDraggable, useDroppable } from "@dnd-kit-svelte/svelte";
  import FolderModal from "./FolderModal.svelte";

  interface Props {
    folder: FavFolderItem;
    idx: number;
    onContextMenu: (event: MouseEvent, itemId: string) => void;
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
    globalState.preselectedItem === folder.itemId || (idx === 0 && !globalState.preselectedItem)
  );

  const folderName = $derived(folder.name || $t("folder"));
  const expandedItems = $derived(
    folder.itemIds.map((id) => ({
      id,
      item: globalState.getMenuItem(id)!,
    }))
  );

  const draggable = useDraggable({
    id: () => folder.itemId,
    type: "folder",
  });

  const droppable = useDroppable({
    id: () => folder.itemId,
    type: "folder",
  });

  function openModal() {
    isModalOpen = true;
  }

  function closeModal() {
    isModalOpen = false;
  }
</script>

<button
  {@attach draggable.ref}
  {@attach droppable.ref}
  data-item-id={folder.itemId}
  class="folder"
  class:preselected={isPreselected && globalState.searchQuery}
  class:is-dragging={draggable.isDragging.current}
  class:is-dropping={draggable.isDropping.current}
  class:is-drop-target={isActiveDropzone}
  onclick={openModal}
  oncontextmenu={(event) => {
    onContextMenu(event, folder.itemId);
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
