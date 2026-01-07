<script lang="ts">
  import { type StartMenuItem } from "@seelen-ui/lib/types";
  import { globalState } from "../state.svelte";
  import { t } from "../i18n";
  import AppItem from "./AppItem.svelte";
  import * as fuzzySearch from "@m31coding/fuzzy-search";
  import { DragDropProvider } from "@dnd-kit-svelte/svelte";

  interface Props {
    onContextMenu: (event: MouseEvent, itemId: string) => void;
  }

  let { onContextMenu }: Props = $props();

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
      (item) => {
        let terms = [item.display_name];
        /* let parts = item.path.split(/[\\/]/g);
        terms.push(parts.slice(-3).join(" ")); */
        return terms;
      }
    );
    return searcher;
  });

  const filteredItems = $derived.by(() => {
    if (globalState.searchQuery) {
      let result = searcher.getMatches(new fuzzySearch.Query(globalState.searchQuery, 21));
      return result.matches.map((match) => match.entity);
    }
    return items;
  });
</script>

<DragDropProvider>
  <div class="all-apps-view">
    <div class="all-apps-view-list">
      {#each filteredItems as item, idx (item.umid || item.path)}
        <AppItem {item} {idx} {onContextMenu} draggable={false} />
      {/each}
    </div>

    {#if filteredItems.length === 0 && globalState.searchQuery.length > 0}
      <div class="all-apps-view-empty">
        {$t("no_matching_items")}
      </div>
    {/if}
  </div>
</DragDropProvider>
