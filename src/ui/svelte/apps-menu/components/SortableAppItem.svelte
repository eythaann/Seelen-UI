<script lang="ts">
  import type { StartMenuItem } from "@seelen-ui/lib/types";
  import { createSortable } from "@dnd-kit/svelte/sortable";
  import AppItem from "./AppItem.svelte";

  interface Props {
    item: StartMenuItem;
    idx: number;
    isActiveDropzone?: boolean;
    isInsideFolder?: boolean;
  }

  let {
    item,
    idx,
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

<AppItem {item} {idx} {isActiveDropzone} {sortable} />
