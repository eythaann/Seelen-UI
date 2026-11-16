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
  import { untrack } from "svelte";
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

  let lastVersion: number | undefined;
  let lastQuery: string | undefined;

  function selectionId(item: SearchResult): string {
    return isSystemSetting(item) ? getResultKey(item) : globalState.getItemId(item);
  }

  $effect(() => {
    const version = globalState.version;
    const query = searchState.searchQuery;
    const items = searchState.searchedItems;
    const selected = untrack(() => selectedItemId);
    const selectedIndex = version === lastVersion && query === lastQuery
      ? items.findIndex((item) => selectionId(item) === selected)
      : -1;
    lastVersion = version;
    lastQuery = query;

    if (selectedIndex === -1) {
      selectedItemId = items[0] ? selectionId(items[0]) : null;
    }

    const chunk = 15;
    // Keep the selected row mounted if it moves past the first chunk.
    let end = Math.max(chunk, selectedIndex + 1);
    let idleCallback = 0;

    function process() {
      visibleItems = items.slice(0, end);

      if (end < items.length) {
        end += chunk;
        idleCallback = requestIdleCallback(process);
      }
    }

    process();
    return () => cancelIdleCallback(idleCallback);
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
