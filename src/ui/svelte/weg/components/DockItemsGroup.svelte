<script lang="ts">
  import type { SwItem } from "../types.ts";
  import DraggableItem from "./DraggableItem.svelte";
  import WegItemSwitch from "./WegItemSwitch.svelte";

  interface Props {
    id: string;
    items: SwItem[];
    itemIndexById: Map<string, number>;
  }

  let { id, items, itemIndexById }: Props = $props();
</script>

<div class="weg-items-{id}" data-empty={items.length === 0}>
  {#each items as item (item.id)}
    <DraggableItem {item} index={itemIndexById.get(item.id) ?? 0}>
      <WegItemSwitch {item} />
    </DraggableItem>
  {/each}
</div>
