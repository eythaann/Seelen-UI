<script lang="ts">
  import type { StartMenuItem } from "@seelen-ui/lib/types";
  import { invoke, SeelenCommand } from "@seelen-ui/lib";
  import { FileIcon } from "libs/ui/svelte/components/Icon";
  import { globalState } from "../state/mod.svelte";
  import { createDroppable, createDraggable } from "@dnd-kit/svelte";

  interface Props {
    item: StartMenuItem;
    idx: number;
    onContextMenu: (event: MouseEvent, item: StartMenuItem) => void;
    draggable?: boolean;
    isActiveDropzone?: boolean;
    isInsideFolder?: boolean;
    lazy?: boolean;
  }

  let {
    item,
    idx,
    onContextMenu,
    draggable: isDraggable = true,
    isActiveDropzone = false,
    isInsideFolder = false,
    lazy = false,
  }: Props = $props();

  const itemId = $derived(item.umid || item.path.toLowerCase());
  const isPreselected = $derived(
    globalState.preselectedItem === itemId || (idx === 0 && !globalState.preselectedItem),
  );

  const draggable = $derived(
    createDraggable({
      id: itemId,
      disabled: !isDraggable,
      type: isInsideFolder ? "grouped-app" : "app",
    }),
  );

  const droppable = $derived(
    createDroppable({
      id: itemId,
      disabled: !isDraggable,
      accept: isInsideFolder ? "grouped-app" : ["folder", "app"],
      type: isInsideFolder ? "grouped-app" : "app",
    }),
  );

  function handleClick(event: MouseEvent) {
    globalState.showing = false; // inmediate close
    let program = item.umid ? `shell:AppsFolder\\${item.umid}` : item.path;
    invoke(SeelenCommand.OpenFile, { path: program });
  }

  function handleContextMenu(event: MouseEvent) {
    onContextMenu(event, item);
  }

  // class:is-dragging={sortableData.isDragging.current}
</script>

<button
  {@attach draggable.attach}
  {@attach droppable.attach}
  data-item-id={itemId}
  class="app"
  class:preselected={isPreselected && globalState.searchQuery}
  class:is-dragging={draggable.isDragging}
  class:is-dropping={draggable.isDropping}
  class:is-drop-target={isActiveDropzone}
  onclick={handleClick}
  oncontextmenu={handleContextMenu}
  onfocus={() => {
    globalState.preselectedItem = itemId;
  }}
>
  <FileIcon class="app-icon" path={item.path} umid={item.umid} {lazy} />
  <div class="app-name" title={item.display_name}>
    {item.display_name}
  </div>
</button>
