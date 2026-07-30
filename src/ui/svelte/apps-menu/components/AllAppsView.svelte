<script lang="ts">
  import { t } from "../i18n";
  import AppItem from "./AppItem.svelte";
  import { searchState, getItemKey } from "../state/search.svelte";
  import { globalState } from "../state/mod.svelte";
  import {
    navigateInDirection,
    selectPreselectedOrFirst,
    type SelectionScope,
  } from "../keyboard-navigation";
  import type { StartMenuItem } from "@seelen-ui/lib/types";

  let visibleItems: StartMenuItem[] = $state([]);

  // Selection is fully local to this view: its own state, its own scope,
  // its own keydown handling. Nothing outside this component touches it.
  let selectedItemId = $state<string | null>(null);

  const scope: SelectionScope = {
    container: () => document,
    getSelected: () => selectedItemId,
    setSelected: (id) => {
      selectedItemId = id;
    },
  };

  // Reset selection whenever the menu is (re)opened or the search query changes.
  // While searching, preselect the first result instead of leaving it empty.
  $effect(() => {
    globalState.version;
    searchState.searchQuery;

    const firstItem = searchState.searchQuery ? searchState.searchedItems[0] : undefined;
    selectedItemId = firstItem ? globalState.getItemId(firstItem) : null;
  });

  function handleWindowKeyDown(event: KeyboardEvent) {
    switch (event.key) {
      case "Enter":
        event.preventDefault();
        selectPreselectedOrFirst(scope)?.click();
        break;
      case "ArrowUp":
        event.preventDefault();
        navigateInDirection("up", scope);
        break;
      case "ArrowDown":
        event.preventDefault();
        navigateInDirection("down", scope);
        break;
      case "ArrowLeft":
        event.preventDefault();
        navigateInDirection("left", scope);
        break;
      case "ArrowRight":
        event.preventDefault();
        navigateInDirection("right", scope);
        break;
    }
  }

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

<svelte:window onkeydown={handleWindowKeyDown} />
<div class="all-apps-view">
  <ul role="listbox" class="all-apps-view-list">
    {#each visibleItems as item, idx (getItemKey(item))}
      <AppItem {item} {idx} lazy {scope} />
    {/each}
  </ul>

  {#if searchState.searchedItems.length === 0}
    <div class="all-apps-view-empty">
      {searchState.searchFilter === "web" ? $t("web_search") : $t("no_matching_items")}
    </div>
  {/if}
</div>
