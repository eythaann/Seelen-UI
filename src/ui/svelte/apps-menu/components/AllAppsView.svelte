<script lang="ts">
  import { type StartMenuItem } from "@seelen-ui/lib/types";
  import { globalState } from "../state/mod.svelte";
  import { foldersAsStartMenuItems } from "../state/knownFolders.svelte";
  import { t } from "../i18n";
  import AppItem from "./AppItem.svelte";
  import * as fuzzySearch from "@m31coding/fuzzy-search";
  import { DragDropProvider } from "@dnd-kit/svelte";

  interface Props {
    onContextMenu: (event: MouseEvent, item: StartMenuItem) => void;
  }

  let { onContextMenu }: Props = $props();

  // Memoized filter function - avoid recreating on each render
  const shouldIncludeItem = (item: StartMenuItem, prefix: string | null): boolean => {
    if (prefix === "web") return false;

    const isApp =
      !!item.umid ||
      item.path?.toLowerCase().endsWith(".exe") ||
      item.path?.toLowerCase().endsWith(".lnk");

    if (prefix === "apps") return isApp;
    if (prefix === "documents") return !isApp;

    return true;
  };

  // Cache filtered/sorted base items - only recalculates when allItems changes
  const items = $derived.by(() => {
    const allItems = globalState.allItems;
    const filtered: StartMenuItem[] = [];

    for (const item of allItems) {
      if (!item.path) {
        if (item.umid) filtered.push(item);
        continue;
      }

      const path = item.path.toLowerCase();
      const lastSlash = Math.max(path.lastIndexOf("\\"), path.lastIndexOf("/"));
      const filename = lastSlash >= 0 ? path.slice(lastSlash + 1) : path;

      if (!filename.includes("uninstall") && filename !== "desktop.ini") {
        filtered.push(item);
      }
    }

    return filtered.sort((a, b) => a.display_name.localeCompare(b.display_name));
  });

  // Parse query only when searchQuery changes
  const query = $derived.by(() => {
    const rawQuery = globalState.searchQuery.trim();
    const prefixMatch = rawQuery.match(/^(apps|files|web):/i);

    if (prefixMatch) {
      const prefix = prefixMatch[1]!.toLowerCase();
      const search = rawQuery.slice(prefix.length + 1).trim();
      return { prefix, search, isSearching: true };
    }

    return { prefix: null, search: rawQuery, isSearching: rawQuery.length > 0 };
  });

  // Only create searcher when items or search state changes - not on every query change
  let cachedSearcher: fuzzySearch.DynamicSearcher<StartMenuItem, string> | null = null;
  let lastSearchableItemsKey = "";

  const filteredItems = $derived.by(() => {
    if (!query.isSearching) {
      return items;
    }

    if (query.prefix === "web") {
      return [];
    }

    // Build searchable items list
    const searchableItems: StartMenuItem[] = [];
    const { prefix } = query;

    for (const item of items) {
      if (shouldIncludeItem(item, prefix)) {
        searchableItems.push(item);
      }
    }

    // Add known folders when searching
    for (const item of foldersAsStartMenuItems.value) {
      if (shouldIncludeItem(item, prefix)) {
        searchableItems.push(item);
      }
    }

    // Create a stable key to check if we need to rebuild the searcher
    const itemsKey = `${items.length}_${foldersAsStartMenuItems.value.length}_${prefix}`;

    if (cachedSearcher === null || lastSearchableItemsKey !== itemsKey) {
      const config = fuzzySearch.Config.createDefaultConfig();
      config.normalizerConfig.allowCharacter = (_c) => true;
      cachedSearcher = fuzzySearch.SearcherFactory.createSearcher<StartMenuItem, string>(config);

      cachedSearcher.indexEntities(
        searchableItems,
        (item) => `${item.path}_${item.umid}`,
        (item) => [item.display_name],
      );

      lastSearchableItemsKey = itemsKey;
    }

    if (!query.search) {
      return searchableItems;
    }

    const result = cachedSearcher.getMatches(new fuzzySearch.Query(query.search, 21));
    return result.matches.map((match) => match.entity);
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
