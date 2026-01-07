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

  const draggableData = useDraggable({
    id: () => folder.itemId,
    type: "folder",
  });

  const dropableData = useDroppable({
    id: () => folder.itemId,
    accept: ["app", "folder"],
    type: "dropzone",
  });

  function openModal() {
    isModalOpen = true;
  }

  function closeModal() {
    isModalOpen = false;
  }
</script>

<button
  {@attach dropableData.ref}
  {@attach draggableData.ref}
  data-item-id={folder.itemId}
  class="folder-item"
  class:preselected={isPreselected && globalState.searchQuery}
  class:is-drop-target={isActiveDropzone}
  onclick={openModal}
  oncontextmenu={(event) => {
    onContextMenu(event, folder.itemId);
  }}
  onfocus={() => {
    globalState.preselectedItem = folder.itemId;
  }}
>
  <div class="folder-item-grid">
    {#each expandedItems.slice(0, 4) as app}
      <div class="folder-item-preview">
        <FileIcon class="folder-item-preview-icon" path={app.item.path} umid={app.item.umid} />
      </div>
    {/each}
  </div>
  <div class="folder-item-name" title={folderName}>
    {folderName}
  </div>
</button>

{#if isModalOpen}
  <FolderModal {folder} onClose={closeModal} {onContextMenu} />
{/if}
