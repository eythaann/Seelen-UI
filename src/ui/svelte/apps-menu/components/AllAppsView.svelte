<script lang="ts">
  import { t } from "../i18n";
  import AppItem from "./AppItem.svelte";
  import AppSettingItem from "./AppSettingItem.svelte";
  import {
    searchState,
    getResultKey,
    isSystemSetting,
    type SearchResult,
  } from "../state/search.svelte";
  import { globalState } from "../state/mod.svelte";
  import { handleGridKeyDown, type SelectionScope } from "../keyboard-navigation";

  let visibleItems: SearchResult[] = $state([]);

  let selectedItemId = $state<string | null>(null);

  const scope: SelectionScope = {
    container: () => document,
    getSelected: () => selectedItemId,
    setSelected: (id) => {
      selectedItemId = id;
    },
  };

  // Preselect the first item when the menu opens or the results change.
  $effect(() => {
    globalState.version;

    const firstItem = searchState.searchedItems[0];
    if (!firstItem) {
      selectedItemId = null;
    } else {
      selectedItemId = isSystemSetting(firstItem)
        ? getResultKey(firstItem)
        : globalState.getItemId(firstItem);
    }
  });

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

<svelte:window onkeydown={(event) => handleGridKeyDown(event, scope)} />
<div class="all-apps-view">
  <ul role="listbox" class="all-apps-view-list">
    {#each visibleItems as item, idx (getResultKey(item))}
      {#if isSystemSetting(item)}
        <AppSettingItem {item} {idx} {scope} />
      {:else}
        <AppItem {item} {idx} lazy {scope} />
      {/if}
    {/each}
  </ul>

  {#if searchState.searchedItems.length === 0}
    <div class="all-apps-view-empty">
      {searchState.searchFilter === "web" ? $t("web_search") : $t("no_matching_items")}
    </div>
  {/if}
</div>
