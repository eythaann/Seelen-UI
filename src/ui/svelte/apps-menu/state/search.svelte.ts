import type { StartMenuItem } from "@seelen-ui/lib/types";
import { Document } from "flexsearch";
import { globalState } from "./mod.svelte";
import { foldersAsStartMenuItems } from "./knownFolders.svelte";

let rawSearch = $state("");

const filterBy = $derived.by(() => {
  const raw = rawSearch.trim().toLowerCase();
  if (raw.startsWith("apps:")) return "apps";
  if (raw.startsWith("files:")) return "files";
  if (raw.startsWith("web:")) return "web";
  return undefined;
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

const hasSearch = $derived(search.length > 0);

interface SearchableItem extends StartMenuItem {
  isApp?: boolean;
}

const _itemsWhereToSearch: SearchableItem[] = $derived.by(() => {
  const allItems: StartMenuItem[] = [...globalState.allItems];

  if (hasSearch) {
    allItems.push(...foldersAsStartMenuItems.value);
  }

  if (filterBy === "web") {
    return [];
  }

  const shouldInclude = {
    apps: !filterBy || filterBy === "apps",
    documents: !filterBy || filterBy === "files",
  };

  const filtered: SearchableItem[] = [];

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
      filtered.push({ ...item, isApp });
    }
  }

  return filtered.toSorted((a, b) => a.display_name.localeCompare(b.display_name));
});

// Rebuilds only when the item list changes, not on every keystroke.
const _searchIndex = $derived.by(() => {
  const itemMap = new Map<string, StartMenuItem>();
  const index = new Document<{
    id: string;
    display_name: string;
    filename?: string;
    initials?: string;
  }>({
    document: { id: "id", index: ["display_name", "filename", "initials"] },
    tokenize: "forward",
  });

  for (const item of _itemsWhereToSearch) {
    const id = getItemKey(item);
    itemMap.set(id, item);

    if (!item.isApp) {
      index.add({ id, display_name: item.display_name });
      continue;
    }

    const path = item.path ?? "";
    const filename = item.target ? getFileStem(item.target) : getFileStem(path);
    const initials = getInitials(item.display_name);

    index.add({ id, display_name: item.display_name, filename, initials });
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
  const matched: SearchableItem[] = [];

  for (const fieldResult of results) {
    for (const id of fieldResult.result as string[]) {
      if (!seen.has(id) && itemMap.has(id)) {
        seen.add(id);
        matched.push(itemMap.get(id)!);
      }
    }
  }

  // apps first, then cap to the global limit
  return matched.toSorted((a, b) => Number(b.isApp) - Number(a.isApp));
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

function getFileStem(p: string): string {
  const s = Math.max(p.lastIndexOf("\\"), p.lastIndexOf("/"));
  return (s >= 0 ? p.slice(s + 1) : p).replace(/\.[^.]+$/, "");
}

function getInitials(s: string) {
  let out = "";
  let take = true;

  for (let i = 0; i < s.length; i++) {
    const c = s[i];

    if (take && c !== " ") {
      out += c;
      take = false;
    }

    if (c === " ") {
      take = true;
    }
  }

  return out.toLowerCase();
}

export const getItemKey = (item: StartMenuItem) => `${item.path}_${item.umid}`;
