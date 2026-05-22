<script lang="ts">
  import { t } from "../i18n";
  import AppItem from "./AppItem.svelte";
  import { searchState, getItemKey } from "../state/search.svelte";
  import type { StartMenuItem } from "@seelen-ui/lib/types";

  let visibleItems: StartMenuItem[] = $state([]);

  $effect(() => {
    visibleItems = [];

    let i = 0;
    const chunk = 15;

    function process() {
      visibleItems = searchState.searchedItems.slice(0, (i += chunk));

      if (i < searchState.searchedItems.length) {
        requestIdleCallback(process);
      }
    }

    process();
  });
</script>

<div class="all-apps-view">
  <div class="all-apps-view-list">
    {#each visibleItems as item, idx (getItemKey(item))}
      <AppItem {item} {idx} lazy />
    {/each}
  </div>

  {#if searchState.searchedItems.length === 0}
    <div class="all-apps-view-empty">
      {searchState.searchFilter === "web" ? $t("web_search") : $t("no_matching_items")}
    </div>
  {/if}
</div>
