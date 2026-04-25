<script lang="ts">
  import type { StartMenuItem } from "@seelen-ui/lib/types";
  import { Widget, invoke, SeelenCommand } from "@seelen-ui/lib";
  import { FileIcon } from "libs/ui/svelte/components/Icon";
  import { globalState } from "../state/mod.svelte";
  import { searchState } from "../state/search.svelte";
  import { t } from "../i18n";
  import type { createSortable } from "@dnd-kit/svelte/sortable";
  import { getItemContextMenu } from "./context-menu.svelte";

  interface Props {
    item: StartMenuItem;
    idx: number;
    isActiveDropzone?: boolean;
    lazy?: boolean;
    sortable?: ReturnType<typeof createSortable> | null;
  }

  let { item, idx, isActiveDropzone = false, lazy = false, sortable = null }: Props = $props();

  const itemId = $derived(item.umid || item.path.toLowerCase());
  const isPreselected = $derived(
    globalState.preselectedItem === itemId || (idx === 0 && !globalState.preselectedItem),
  );

  const menu = $derived(getItemContextMenu(item, $t));
  const noopAttach = () => {};

  function handleClick() {
    Widget.self.hide();
    const program = item.umid ? `shell:AppsFolder\\${item.umid}` : item.path;
    invoke(SeelenCommand.OpenFile, { path: program });
  }

  function handleContextMenu() {
    invoke(SeelenCommand.TriggerContextMenu, {
      menu,
      forwardTo: null,
    });
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
