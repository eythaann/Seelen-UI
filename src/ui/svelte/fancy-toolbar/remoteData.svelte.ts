import { untrack } from "svelte";
import type { ToolbarItem } from "@seelen-ui/lib/types";

export function createRemoteDataResolver(
  getRemoteData: () => ToolbarItem["remoteData"],
): Record<string, any> {
  let fetchedData = $state<Record<string, any>>({});

  $effect(() => {
    const intervals: Record<string, ReturnType<typeof setInterval>> = {};
    let mounted = true;

    async function fetchKey(key: string, rd: any) {
      if (!mounted) return;
      try {
        const response = await fetch(rd.url, rd.requestInit as RequestInit);
        const data = response.headers.get("Content-Type")?.includes("application/json")
          ? await response.json()
          : await response.text();
        if (mounted) {
          // `set()` rejects `$state` writes made while the active reaction is a
          // derived/block/eager reaction (see `set()` in Svelte's `sources.js`),
          // and the toolbar flushes those constantly. A fetch completion is a
          // microtask that can land inside such a context — the write then
          // throws `state_unsafe_mutation` and the payload never reaches the
          // script scope. This write reads no reactive state, so run it
          // untracked to escape that context.
          untrack(() => {
            fetchedData[key] = data;
          });
        }
      } catch (err) {
        console.error(`Error fetching ${key}:`, err);
      }
    }

    for (const [key, rd] of Object.entries(getRemoteData())) {
      if (!rd) continue;
      fetchKey(key, rd);
      if ((rd as any).updateIntervalSeconds) {
        intervals[key] = setInterval(
          () => fetchKey(key, rd),
          (rd as any).updateIntervalSeconds * 1000,
        );
      }
    }

    return () => {
      mounted = false;
      Object.values(intervals).forEach(clearInterval);
    };
  });

  return fetchedData;
}
