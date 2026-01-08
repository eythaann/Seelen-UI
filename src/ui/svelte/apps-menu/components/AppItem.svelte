<script lang="ts">
  import type { StartMenuItem } from "@seelen-ui/lib/types";
  import { invoke, SeelenCommand } from "@seelen-ui/lib";
  import { FileIcon } from "libs/ui/svelte/components/Icon";
  import { globalState } from "../state.svelte";
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
    draggable: isDraggable = true,
    isActiveDropzone = false,
    isInsideFolder = false,
  }: Props = $props();

  const itemId = $derived(item.umid || item.path);
  const isPreselected = $derived(
    globalState.preselectedItem === itemId || (idx === 0 && !globalState.preselectedItem)
  );

  const draggable = useDraggable({
    id: () => itemId,
    disabled: () => !isDraggable,
    type: () => (isInsideFolder ? "grouped-app" : "app"),
  });

  const droppable = useDroppable({
    id: () => itemId,
    disabled: () => !isDraggable,
    accept: () => (isInsideFolder ? "grouped-app" : ["folder", "app"]),
    type: () => (isInsideFolder ? "grouped-app" : "app"),
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
  {@attach draggable.ref}
  {@attach droppable.ref}
  data-item-id={itemId}
  class="app"
  class:preselected={isPreselected && globalState.searchQuery}
  class:is-dragging={draggable.isDragging.current}
  class:is-dropping={draggable.isDropping.current}
  class:is-drop-target={isActiveDropzone}
  onclick={handleClick}
  oncontextmenu={handleContextMenu}
  onfocus={() => {
    globalState.preselectedItem = itemId;
  }}
>
  <FileIcon class="app-icon" path={item.path} umid={item.umid} />
  <div class="app-name" title={item.display_name}>
    {item.display_name}
  </div>
</button>
