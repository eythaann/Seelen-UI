import type { RemoteDataDeclaration } from "@seelen-ui/lib/types";
import { useDeepCompareEffect } from "@shared/hooks";
import { useRef, useState } from "preact/compat";

/**
 * Hook to fetch and manage remote data with optional periodic updates.
 * @param remoteData - Object containing remote data declarations
 * @returns Object with fetched data for each key
 */
export function useRemoteData(
  remoteData: Record<string, RemoteDataDeclaration | undefined>,
) {
  const [state, setState] = useState<Record<string, any>>(() => {
    return Object.keys(remoteData).reduce((acc, key) => ({ ...acc, [key]: undefined }), {});
  });

  const intervalsRef = useRef<Record<string, ReturnType<typeof setInterval>>>({});
  const mountedRef = useRef(true);

  const fetchData = async (key: string, rd: RemoteDataDeclaration): Promise<void> => {
    if (!mountedRef.current) return;

    try {
      const response = await fetch(rd.url, rd.requestInit as RequestInit);
      const data = response.headers.get("Content-Type")?.includes("application/json")
        ? await response.json()
        : await response.text();

      if (mountedRef.current) {
        setState((prev) => ({
          ...prev,
          [key]: data,
        }));
      }
    } catch (error) {
      console.error(`Error fetching ${key}:`, error);
    }
  };

  useDeepCompareEffect(() => {
    mountedRef.current = true;
    Object.values(intervalsRef.current).forEach(clearInterval);
    intervalsRef.current = {};

    const initialState = Object.keys(remoteData).reduce(
      (acc, key) => ({ ...acc, [key]: undefined }),
      {},
    );

    setState((prev) => ({ ...initialState, ...prev }));

    Object.entries(remoteData).forEach(([key, rd]) => {
      if (!rd) return;
      fetchData(key, rd);
      if (rd.updateIntervalSeconds) {
        intervalsRef.current[key] = globalThis.setInterval(
          () => fetchData(key, rd),
          rd.updateIntervalSeconds * 1000,
        );
      }
    });

    return () => {
      mountedRef.current = false;
      Object.values(intervalsRef.current).forEach(clearInterval);
    };
  }, [remoteData]);

  return state;
}
