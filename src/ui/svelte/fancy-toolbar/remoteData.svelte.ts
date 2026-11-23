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
          // Item components (and this resolver) are created inside ItemsGroup's
          // derived `entries`, and the toolbar flushes constantly (clock ticks,
          // scope updates). When a fetch completion lands while such a flush is
          // in progress, the current reaction is a derived and this write gets
          // rejected with `state_unsafe_mutation`, so the payload never reaches
          // the script scope and the item renders `undefined` forever. The write
          // is intentionally untracked: it reads no reactive state.
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
