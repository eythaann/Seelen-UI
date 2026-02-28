<script lang="ts">
  import type { StartMenuItem } from "@seelen-ui/lib/types";
  import { invoke, SeelenCommand } from "@seelen-ui/lib";
  import { FileIcon } from "libs/ui/svelte/components/Icon";
  import { globalState } from "../state/mod.svelte";
  import { createSortable } from "@dnd-kit/svelte/sortable";

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

  const sortable = createSortable({
    get id() {
      return itemId;
    },
    get disabled() {
      return !isDraggable;
    },
    get index() {
      return idx;
    },
    get type() {
      return isInsideFolder ? "grouped-app" : "app";
    },
    get accept() {
      return isInsideFolder ? "grouped-app" : ["folder", "app"];
    },
  });

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
  {@attach sortable.attach}
  data-item-id={itemId}
  class="app"
  class:preselected={isPreselected && globalState.searchQuery}
  class:is-dragging={sortable.isDragging}
  class:is-dropping={sortable.isDropping}
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
