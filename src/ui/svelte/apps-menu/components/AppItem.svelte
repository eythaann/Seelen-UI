<script lang="ts">
  import type { StartMenuItem } from "@seelen-ui/lib/types";
  import { invoke, SeelenCommand } from "@seelen-ui/lib";
  import { FileIcon } from "libs/ui/svelte/components/Icon";
  import { globalState } from "../state.svelte";
  import { useSortable } from "@dnd-kit-svelte/svelte/sortable";
  import { useDraggable, useDroppable } from "@dnd-kit-svelte/svelte";

  interface Props {
    item: StartMenuItem;
    idx: number;
    onContextMenu: (event: MouseEvent, itemId: string) => void;
    draggable?: boolean;
    isActiveDropzone?: boolean;
    isInsideFolder?: boolean;
  }

  let {
    item,
    idx,
    onContextMenu,
    draggable = true,
    isActiveDropzone = false,
    isInsideFolder = false,
  }: Props = $props();

  const itemId = $derived(item.umid || item.path);
  const isPreselected = $derived(
    globalState.preselectedItem === itemId || (idx === 0 && !globalState.preselectedItem)
  );

  /* const sortableData = useSortable({
    id: () => itemId,
    index: () => idx,
    disabled: () => !sortable || true,
    type: "app",
  }); */

  const draggableData = useDraggable({
    id: () => itemId,
    disabled: () => !draggable,
    type: () => isInsideFolder ? "folder-item" : "app",
  });

  const dropableData = useDroppable({
    id: () => itemId,
    accept: ["app", "folder"],
    type: "dropzone",
    disabled: () => isInsideFolder,
  });

  function handleClick(event: MouseEvent) {
    globalState.showing = false; // inmediate close
    if (item.umid) {
      invoke(SeelenCommand.OpenFile, { path: `shell:AppsFolder\\${item.umid}` });
    } else if (item.path) {
      invoke(SeelenCommand.OpenFile, { path: item.path });
    }
  }

  function handleContextMenu(event: MouseEvent) {
    onContextMenu(event, itemId);
  }

  // class:is-dragging={sortableData.isDragging.current}
</script>

<button
  {@attach dropableData.ref}
  {@attach draggableData.ref}
  data-item-id={itemId}
  class="app-item"
  class:preselected={isPreselected && globalState.searchQuery}
  class:is-drop-target={isActiveDropzone}
  onclick={handleClick}
  oncontextmenu={handleContextMenu}
  onfocus={() => {
    globalState.preselectedItem = itemId;
  }}
>
  <FileIcon class="app-item-icon" path={item.path} umid={item.umid} />
  <div class="app-item-name" title={item.display_name}>
    {item.display_name}
  </div>
</button>
