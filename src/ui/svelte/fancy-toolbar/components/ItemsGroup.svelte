<script lang="ts">
  import { flip } from "svelte/animate";
  import { scale } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import type { ToolbarItem, ToolbarItem2 } from "@seelen-ui/lib/types";
  import { plugins, toolbarState } from "../state/items.svelte.ts";
  import SortableItem from "./SortableItem.svelte";

  interface Props {
    id: string;
    items: ToolbarItem2[];
    itemIndexById: Map<string, number>;
  }

  let { id, items, itemIndexById }: Props = $props();

</script>

<div class="ft-bar-container ft-bar-{id}">
  {#each items as entry (typeof entry === "string" ? entry : entry.id)}
    {@const index = itemIndexById.get(typeof entry === "string" ? entry : entry.id) ?? 0}
    <div
      class="ft-bar-item-animator"
      animate:flip={{ duration: 200, easing: cubicOut }}
      transition:scale={{ duration: 180, start: 0.7, easing: cubicOut }}
    >
      {#if typeof entry === "string"}
        {@const cached = plugins.value.find((p) => p.id === entry)}
        {#if cached}
          {@const module = { ...(cached.plugin as ToolbarItem), id: entry }}
          <SortableItem {module} {index} pluginId={entry} />
        {/if}
      {:else}
        <SortableItem module={entry} {index} />
      {/if}
    </div>
  {/each}
</div>
