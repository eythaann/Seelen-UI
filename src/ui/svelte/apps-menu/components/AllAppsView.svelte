<script lang="ts">
  import { type StartMenuItem } from "@seelen-ui/lib/types";
  import { globalState } from "../state.svelte";
  import { t } from "../i18n";
  import AppItem from "./AppItem.svelte";
  import * as fuzzySearch from "@m31coding/fuzzy-search";

  // this only will change when start items change
  const items = $derived.by(() => {
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

  let searcher = $derived.by(() => {
    const config = fuzzySearch.Config.createDefaultConfig();
    config.normalizerConfig.allowCharacter = (_c) => true;
    const searcher = fuzzySearch.SearcherFactory.createSearcher<StartMenuItem, string>(config);

    searcher.indexEntities(
      items,
      (item) => `${item.path}_${item.umid}`,
      (item) => [item.display_name, item.path, item.umid || ""]
    );
    return searcher;
  });

  const filteredItems = $derived.by(() => {
    if (globalState.searchQuery) {
      let result = searcher.getMatches(new fuzzySearch.Query(globalState.searchQuery));
      return result.matches.map((match) => match.entity);
    }
    return items;
  });
</script>

<div class="all-apps-view">
  <div class="all-apps-view-list">
    {#each filteredItems as item, idx (item.umid || item.path)}
      <AppItem {item} {idx} class="all-apps-view-item" />
    {/each}
  </div>

  {#if filteredItems.length === 0 && globalState.searchQuery.length > 0}
    <div class="all-apps-view-empty">
      {$t("no_matching_items")}
    </div>
  {/if}
</div>
