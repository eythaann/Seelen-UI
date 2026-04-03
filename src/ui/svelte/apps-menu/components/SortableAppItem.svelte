<script lang="ts">
  import type { StartMenuItem } from "@seelen-ui/lib/types";
  import { createSortable } from "@dnd-kit/svelte/sortable";
  import AppItem from "./AppItem.svelte";

  interface Props {
    item: StartMenuItem;
    idx: number;
    onContextMenu: (event: MouseEvent, item: StartMenuItem) => void;
    isActiveDropzone?: boolean;
    isInsideFolder?: boolean;
  }

  let {
    item,
    idx,
    onContextMenu,
    isActiveDropzone = false,
    isInsideFolder = false,
  }: Props = $props();

  const itemId = $derived(item.umid || item.path.toLowerCase());

  const sortable = createSortable({
    get id() {
      return itemId;
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
</script>

<AppItem {item} {idx} {onContextMenu} {isActiveDropzone} {sortable} />
