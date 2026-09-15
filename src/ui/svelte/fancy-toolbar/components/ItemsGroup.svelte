<script lang="ts">
  import type { ToolbarItem, ToolbarItem2 } from "@seelen-ui/lib/types";
  import { plugins } from "../state/items.svelte.ts";
  import SortableItem from "./SortableItem.svelte";

  interface Props {
    id: string;
    items: ToolbarItem2[];
    itemIndexById: Map<string, number>;
  }

  let { id, items, itemIndexById }: Props = $props();

  let entries = $derived.by(() => {
    const expanded: { pluginId?: string; index: number; item: ToolbarItem }[] = [];
    for (const entry of items) {
      if (typeof entry !== "string") {
        const index = itemIndexById.get(entry.id) ?? 0;
        expanded.push({ item: entry, index });
        continue;
      }

      const index = itemIndexById.get(entry) ?? 0;
      const plugin = plugins.value.find((p) => p.id === entry);
      if (plugin) {
        expanded.push({
          item: { ...(plugin.plugin as ToolbarItem), id: plugin.id },
          pluginId: plugin.id,
          index,
        });
      }
    }
    return expanded;
  });
</script>

<div class="ft-bar-container ft-bar-{id}">
  {#each entries as entry (entry.item.id)}
    <SortableItem module={entry.item} index={entry.index} pluginId={entry.pluginId} />
  {/each}
</div>
