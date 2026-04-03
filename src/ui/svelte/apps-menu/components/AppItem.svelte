<script lang="ts">
  import type { StartMenuItem } from "@seelen-ui/lib/types";
  import { invoke, SeelenCommand } from "@seelen-ui/lib";
  import { FileIcon } from "libs/ui/svelte/components/Icon";
  import { globalState } from "../state/mod.svelte";
  import { searchState } from "../state/search.svelte";
  import type { createSortable } from "@dnd-kit/svelte/sortable";

  interface Props {
    item: StartMenuItem;
    idx: number;
    onContextMenu: (event: MouseEvent, item: StartMenuItem) => void;
    isActiveDropzone?: boolean;
    lazy?: boolean;
    sortable?: ReturnType<typeof createSortable> | null;
  }

  let {
    item,
    idx,
    onContextMenu,
    isActiveDropzone = false,
    lazy = false,
    sortable = null,
  }: Props = $props();

  const itemId = $derived(item.umid || item.path.toLowerCase());
  const isPreselected = $derived(
    globalState.preselectedItem === itemId || (idx === 0 && !globalState.preselectedItem),
  );

  const noopAttach = () => {};

  function handleClick() {
    globalState.showing = false;
    let program = item.umid ? `shell:AppsFolder\\${item.umid}` : item.path;
    invoke(SeelenCommand.OpenFile, { path: program });
  }

  function handleContextMenu(event: MouseEvent) {
    onContextMenu(event, item);
  }
</script>

<button
  {@attach sortable?.attach ?? noopAttach}
  data-item-id={itemId}
  class="app"
  class:preselected={isPreselected && searchState.searchQuery}
  class:is-dragging={sortable?.isDragging}
  class:is-dropping={sortable?.isDropping}
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
