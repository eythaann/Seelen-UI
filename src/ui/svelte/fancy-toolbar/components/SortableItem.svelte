<script lang="ts">
  import type { ToolbarItem } from "@seelen-ui/lib/types";
  import { createSortable } from "@dnd-kit/svelte/sortable";
  import { RestrictToHorizontalAxis } from "@dnd-kit/abstract/modifiers";
  import { toolbarState } from "../state/items.svelte.ts";
  import Item from "./Item.svelte";

  interface Props {
    module: ToolbarItem;
    index: number;
  }

  let { module, index }: Props = $props();

  const sortable = createSortable({
    get id() {
      return module.id;
    },
    get index() {
      return index;
    },
    get disabled() {
      return toolbarState.isReorderDisabled;
    },
    modifiers: [RestrictToHorizontalAxis],
  });
</script>

<Item {module} {sortable} />
