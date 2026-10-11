import type { StartMenuItem, SystemSettingEntry } from "@seelen-ui/lib/types";
import { Document } from "flexsearch";
import { globalState } from "./mod.svelte";
import { foldersAsStartMenuItems } from "./knownFolders.svelte";

const APPS_AND_FILES_LIMIT = 21;
/** Settings shown together with apps and files, kept low to not bury the apps results. */
const SETTINGS_LIMIT_MIXED = 8;
/** Settings shown when filtering by `settings:`, they are the only results. */
const SETTINGS_LIMIT_FILTERED = 50;

let rawSearch = $state("");

const filterBy = $derived.by(() => {
  const raw = rawSearch.trim().toLowerCase();
  if (raw.startsWith("apps:")) return "apps";
  if (raw.startsWith("files:")) return "files";
  if (raw.startsWith("settings:")) return "settings";
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

export type SearchResult = SearchableItem | SystemSettingEntry;

const _itemsWhereToSearch: SearchableItem[] = $derived.by(() => {
  const allItems: StartMenuItem[] = [
    ...globalState.allItems,
    ...(hasSearch ? foldersAsStartMenuItems.value : []),
  ];

  if (filterBy === "web" || filterBy === "settings") {
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
        filtered.push({ ...item, isApp: true });
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

// Built once, the settings list is static per session.
const _settingsIndex = $derived.by(() => {
  const itemMap = new Map<string, SystemSettingEntry>();
  const index = new Document<{ id: string; title: string; keywords: string }>({
    document: { id: "id", index: ["title", "keywords"] },
    // "full" matches in the middle of words, needed for languages without spaces (e.g. chinese)
    tokenize: "full",
  });

  for (const entry of globalState.systemSettings) {
    const id = getResultKey(entry);
    itemMap.set(id, entry);
    index.add({ id, title: entry.title, keywords: entry.keywords.join(" ") });
  }

  return { index, itemMap };
});

const searchedSettings = $derived.by(() => {
  if (filterBy && filterBy !== "settings") {
    return [];
  }

  const limit = filterBy === "settings" ? SETTINGS_LIMIT_FILTERED : SETTINGS_LIMIT_MIXED;
  if (!search.length) {
    // without filter the default "all apps" list is shown, with `settings:` all the entries
    // (settings and control panel), rendered progressively by the view
    return filterBy ? globalState.systemSettings.toSorted((a, b) => a.title.localeCompare(b.title)) : [];
  }

  const { index, itemMap } = _settingsIndex;
  const results = index.search(search, { limit, enrich: false });

  // title matches are returned before keyword matches
  const seen = new Set<string>();
  const matched: SystemSettingEntry[] = [];
  for (const fieldResult of results) {
    for (const id of fieldResult.result as string[]) {
      if (!seen.has(id) && itemMap.has(id)) {
        seen.add(id);
        matched.push(itemMap.get(id)!);
      }
    }
  }
  return matched.slice(0, limit);
});

// Only queries the pre-built index on each keystroke — no indexing here.
const searchedItems = $derived.by((): SearchResult[] => {
  if (!search.length) {
    return [..._itemsWhereToSearch, ...searchedSettings];
  }

  const { index, itemMap } = _searchIndex;
  const results = index.search(search, { limit: APPS_AND_FILES_LIMIT, enrich: false });

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

  // exact settings names first, then apps, then the rest of the items
  const query = search.toLowerCase();
  const exactSettings = searchedSettings.filter((s) => s.title.toLowerCase() === query);
  const otherSettings = searchedSettings.filter((s) => s.title.toLowerCase() !== query);
  const apps = matched.filter((item) => item.isApp);
  const others = matched.filter((item) => !item.isApp);

  return [...exactSettings, ...apps, ...otherSettings, ...others];
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

export function isSystemSetting(item: SearchResult): item is SystemSettingEntry {
  return "kind" in item;
}

/** Unique key of a search result, also used as selection id. */
export function getResultKey(item: SearchResult): string {
  if (isSystemSetting(item)) {
    return `${item.target}_${item.title}`;
  }
  return getItemKey(item);
}
