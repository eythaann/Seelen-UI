<script lang="ts">
  import { type StartMenuItem } from "@seelen-ui/lib/types";
  import { globalState } from "../state/mod.svelte";
  import { foldersAsStartMenuItems } from "../state/knownFolders.svelte";
  import { t } from "../i18n";
  import AppItem from "./AppItem.svelte";
  import * as fuzzySearch from "@m31coding/fuzzy-search";
  import { DragDropProvider } from "@dnd-kit-svelte/svelte";

  interface Props {
    onContextMenu: (event: MouseEvent, item: StartMenuItem) => void;
  }

  let { onContextMenu }: Props = $props();

  // Parse search query to extract prefix and actual query
  const query = $derived.by(() => {
    const query = globalState.searchQuery.trim();
    const prefixMatch = query.match(/^(apps|files|web):/i);

    if (prefixMatch) {
      const prefix = prefixMatch[1]?.toLowerCase() || "";
      const actualQuery = query.slice(prefix.length + 1).trim();
      return { prefix, search: actualQuery, isSearching: true };
    }

    return { prefix: null, search: query, isSearching: query.length > 0 };
  });

  // Filter function based on prefix
  function shouldIncludeItem(item: StartMenuItem, prefix: string | null): boolean {
    if (prefix === "web") {
      return false; // web: doesn't show any items
    }

    const isApp =
      !!item.umid ||
      item.path?.toLowerCase().endsWith(".exe") ||
      item.path?.toLowerCase().endsWith(".lnk");

    if (prefix === "apps") {
      return isApp;
    }

    if (prefix === "documents") {
      return !isApp;
    }

    return true; // no prefix, include all
  }

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

  // Combined items including known folders when searching
  const searchableItems = $derived.by(() => {
    if (query.prefix === "web") {
      return [];
    }
    if (query.isSearching) {
      return [
        ...items.filter((item) => shouldIncludeItem(item, query.prefix)),
        ...foldersAsStartMenuItems.value.filter((item) => shouldIncludeItem(item, query.prefix)),
      ];
    }
    return items;
  });

  let searcher = $derived.by(() => {
    const config = fuzzySearch.Config.createDefaultConfig();
    config.normalizerConfig.allowCharacter = (_c) => true;
    const searcher = fuzzySearch.SearcherFactory.createSearcher<StartMenuItem, string>(config);

    searcher.indexEntities(
      searchableItems,
      (item) => `${item.path}_${item.umid}`,
      (item) => [item.display_name],
    );
    return searcher;
  });

  const filteredItems = $derived.by(() => {
    if (query.isSearching) {
      let result = searcher.getMatches(new fuzzySearch.Query(query.search, 21));
      return result.matches.map((match) => match.entity);
    }
    return items;
  });
</script>

<DragDropProvider>
  <div class="all-apps-view">
    <div class="all-apps-view-list">
      {#each filteredItems as item, idx (item.umid || item.path)}
        <AppItem {item} {idx} {onContextMenu} draggable={false} lazy />
      {/each}
    </div>

    {#if filteredItems.length === 0 && (query.search.length > 0 || query.prefix === "web")}
      <div class="all-apps-view-empty">
        {query.prefix === "web" ? $t("web_search") : $t("no_matching_items")}
      </div>
    {/if}
  </div>
</DragDropProvider>
