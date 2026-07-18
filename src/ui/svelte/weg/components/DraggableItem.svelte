<script lang="ts">
  import type { Snippet } from "svelte";
  import type { SwItem } from "../types.ts";
  import { isHorizontalDock } from "../state/settings.svelte.ts";
  import { createSortable } from "@dnd-kit/svelte/sortable";
  import { RestrictToHorizontalAxis, RestrictToVerticalAxis } from "@dnd-kit/abstract/modifiers";
  import { dockState } from "../state/items.svelte.ts";

  interface Props {
    item: SwItem;
    index: number;
    children: Snippet;
  }

  let { item, index, children }: Props = $props();

  const sortable = createSortable({
    get id() {
      return item.id;
    },
    get index() {
      return index;
    },
    get disabled() {
      return dockState.isReorderDisabled;
    },
    get modifiers() {
      return [isHorizontalDock() ? RestrictToHorizontalAxis : RestrictToVerticalAxis];
    },
  });
</script>

<div
  {@attach sortable.attach}
  style="opacity: {sortable.isDragging ? 0.3 : 1}"
  data-dragging={sortable.isDragging}
  data-item-id={item.id}
  class="weg-item-drag-container"
  class:dragging={sortable.isDragging}
>
  {@render children()}
</div>
