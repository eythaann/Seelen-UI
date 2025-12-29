<script lang="ts">
  import { globalState } from "../state.svelte";
  import AppItem from "./AppItem.svelte";

  const sortedItems = $derived.by(() => {
    return [...globalState.allItems]
      .filter((item) => {
        if (!item.path) return !!item.umid;

        let path = item.path.toLowerCase();
        let filename = path.split(/[\\/]/g).pop() || "";
        // hide uninstallers and desktop.ini files
        return !filename.includes("uninstall") && filename !== "desktop.ini";
      })
      .sort((a, b) => a.display_name.localeCompare(b.display_name));
  });
</script>

<div class="all-apps-view">
  <div class="all-apps-view-list">
    {#each sortedItems as item (item.umid || item.path)}
      <AppItem {item} class="all-apps-view-item" />
    {/each}
  </div>
</div>
