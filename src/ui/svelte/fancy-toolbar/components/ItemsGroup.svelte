<script lang="ts">
  import type { ToolbarItem, ToolbarItem2 } from "@seelen-ui/lib/types";
  import { plugins } from "../state/items.svelte.ts";
  import SortableItem from "./SortableItem.svelte";

  interface Props {
    id: string;
    items: ToolbarItem2[];
    startIndex: number;
  }

  let { id, items, startIndex }: Props = $props();
</script>

<div class="ft-bar-container ft-bar-{id}">
  {#each items as entry, localIndex (typeof entry === "string" ? entry : entry.id)}
    {@const index = startIndex + localIndex}
    {#if typeof entry === "string"}
      {@const cached = plugins.value.find((p) => p.id === entry)}
      {#if cached}
        {@const module = { ...(cached.plugin as ToolbarItem), id: entry }}
        <SortableItem {module} {index} />
      {/if}
    {:else}
      <SortableItem module={entry} {index} />
    {/if}
  {/each}
</div>
