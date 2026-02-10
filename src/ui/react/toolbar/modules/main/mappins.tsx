import type { ToolbarItem, ToolbarItem2 } from "@seelen-ui/lib/types";
import { effect } from "@preact/signals";
import { memo } from "preact/compat";

import { $plugins } from "../shared/state/items.ts";
import { Item } from "../item/infra/infra.tsx";

// Memoized wrapper for Item to prevent unnecessary re-renders
const MemoizedItem = memo(Item);

// Cache for resolved plugin modules to avoid repeated lookups
// Key: plugin ID, Value: resolved module or null if not found
const pluginCache = new Map<string, ToolbarItem | null>();

// Clear cache when plugins change to ensure fresh lookups
effect(() => {
  // Read $plugins.value to track changes
  $plugins.value;
  // Clear cache on any change
  pluginCache.clear();
});

// item can be a toolbar plugin id or a toolbar module
// Optimized with caching to reduce plugin lookups (O(n) -> O(1) for repeated items)
export function componentByModule(entry: ToolbarItem2) {
  let module: ToolbarItem | undefined;

  if (typeof entry === "string") {
    // Check cache first
    if (pluginCache.has(entry)) {
      const cached = pluginCache.get(entry);
      if (!cached) {
        return null;
      }
      module = { ...cached, id: entry };
    } else {
      // Lookup and cache
      const found = $plugins.value.find((p) => p.id === entry)?.plugin as ToolbarItem | undefined;
      pluginCache.set(entry, found || null);

      if (!found) {
        return null;
      }

      module = { ...found, id: entry };
    }
  } else {
    module = entry;
  }

  return <MemoizedItem key={module.id} module={module} />;
}
