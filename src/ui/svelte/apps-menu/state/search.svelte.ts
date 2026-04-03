import type { StartMenuItem } from "@seelen-ui/lib/types";
import { Document } from "flexsearch";
import { globalState } from "./mod.svelte";
import { foldersAsStartMenuItems } from "./knownFolders.svelte";

let rawSearch = $state("");

const filterBy = $derived.by(() => {
  const rawQuery = rawSearch.trim();
  const prefixMatch = rawQuery.match(/^(apps|files|web):/i);
  return prefixMatch?.[1]?.toLowerCase();
});

const search = $derived.by(() => {
  if (filterBy) {
    return rawSearch
      .trimStart()
      .slice(filterBy.length + 1)
      .trim();
  }
  return rawSearch.trim();
});

const _itemsWhereToSearch = $derived.by(() => {
  const allItems: StartMenuItem[] = [...globalState.allItems];

  if (search.length) {
    allItems.push(...foldersAsStartMenuItems.value);
  }

  if (filterBy === "web") {
    return [];
  }

  const shouldInclude = {
    apps: !filterBy || filterBy === "apps",
    documents: !filterBy || filterBy === "files",
  };

  const filtered: StartMenuItem[] = [];

  for (const item of allItems) {
    if (!item.path) {
      if (item.umid && shouldInclude.apps) {
        filtered.push(item);
      }
      continue;
    }

    const path = item.path.toLowerCase();
    const isApp = !!item.umid || path.endsWith(".exe") || path.endsWith(".lnk");

    if (isApp && !shouldInclude.apps) {
      continue;
    }

    if (!isApp && !shouldInclude.documents) {
      continue;
    }

    const lastSlash = Math.max(path.lastIndexOf("\\"), path.lastIndexOf("/"));
    const filename = lastSlash >= 0 ? path.slice(lastSlash + 1) : path;
    if (!filename.includes("uninstall") && filename !== "desktop.ini") {
      filtered.push(item);
    }
  }

  return filtered.sort((a, b) => a.display_name.localeCompare(b.display_name));
});

// Rebuilds only when the item list changes, not on every keystroke.
const _searchIndex = $derived.by(() => {
  const itemMap = new Map<string, StartMenuItem>();
  const index = new Document<{ id: string; display_name: string }>({
    document: { id: "id", index: ["display_name"] },
    tokenize: "forward",
  });

  for (const item of _itemsWhereToSearch) {
    const id = getItemKey(item);
    itemMap.set(id, item);
    index.add({ id, display_name: item.display_name });
  }

  return { index, itemMap };
});

// Only queries the pre-built index on each keystroke — no indexing here.
const searchedItems = $derived.by(() => {
  if (!search.length) {
    return _itemsWhereToSearch;
  }

  const { index, itemMap } = _searchIndex;
  const results = index.search(search, { limit: 21, enrich: false });

  const seen = new Set<string>();
  const matched: StartMenuItem[] = [];

  for (const fieldResult of results) {
    for (const id of fieldResult.result as string[]) {
      if (!seen.has(id) && itemMap.has(id)) {
        seen.add(id);
        matched.push(itemMap.get(id)!);
      }
    }
  }

  // apps first
  return matched.sort((a, b) => {
    let aIsApp = isApp(a) ? 1 : 0;
    let bIsApp = isApp(b) ? 1 : 0;
    return bIsApp - aIsApp;
  });
});

export const searchState = {
  get searchQuery() {
    return rawSearch;
  },

  set searchQuery(value: string) {
    rawSearch = value;
  },

  get searchFilter() {
    return filterBy;
  },

  get searchedItems() {
    return searchedItems;
  },
};

function isApp(item: StartMenuItem) {
  return !!item.umid || item.path.endsWith(".exe") || item.path.endsWith(".lnk");
}
export const getItemKey = (item: StartMenuItem) => `${item.path}_${item.umid}`;
