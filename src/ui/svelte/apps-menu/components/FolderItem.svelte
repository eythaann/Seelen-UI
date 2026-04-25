<script lang="ts">
  import type { FavFolderItem } from "../state/mod.svelte";
  import { FileIcon } from "libs/ui/svelte/components/Icon";
  import { globalState } from "../state/mod.svelte";
  import { searchState } from "../state/search.svelte";
  import { t } from "../i18n";
  import { invoke, SeelenCommand } from "@seelen-ui/lib";
  import { createSortable } from "@dnd-kit/svelte/sortable";
  import FolderModal from "./FolderModal.svelte";
  import { getFolderContextMenu } from "./context-menu.svelte";

  interface Props {
    folder: FavFolderItem;
    idx: number;
    isActiveDropzone?: boolean;
  }

  let { folder, idx, isActiveDropzone = false }: Props = $props();

  const menu = $derived(getFolderContextMenu(folder, $t));

  function handleFolderContextMenu() {
    invoke(SeelenCommand.TriggerContextMenu, {
      menu,
      forwardTo: null,
    });
  }

  let isModalOpen = $state(false);
  $effect(() => {
    globalState.version; // re-run when version changes to reset modal state
    isModalOpen = false;
  });

  const isPreselected = $derived(
    globalState.preselectedItem === folder.itemId || (idx === 0 && !globalState.preselectedItem),
  );

  const folderName = $derived(folder.name || $t("folder"));
  const expandedItems = $derived(
    folder.itemIds
      .map((id) => ({
        id,
        item: globalState.getMenuItem(id)!,
      }))
      .filter((app) => !!app.item),
  );

  const sortable = createSortable({
    get id() {
      return folder.itemId;
    },
    get index() {
      return idx;
    },
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
  {@attach sortable.attach}
  data-item-id={folder.itemId}
  class="folder"
  class:preselected={isPreselected && searchState.searchQuery}
  class:is-dragging={sortable.isDragging}
  class:is-dropping={sortable.isDropping}
  class:is-drop-target={isActiveDropzone}
  onclick={openModal}
  oncontextmenu={handleFolderContextMenu}
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
  <FolderModal {folder} onClose={closeModal} />
{/if}
