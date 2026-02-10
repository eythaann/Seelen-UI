import { type AllSeelenCommandReturns, invoke, SeelenCommand, SeelenEvent, subscribe } from "@seelen-ui/lib";
import { useEffect, useState } from "preact/hooks";

export interface Scope<T> {
  fetching: boolean;
  data: T | null;
}

interface LazyStateConfig<C extends SeelenCommand> {
  command: C;
  event: SeelenEvent;
}

/**
 * Hook to create a lazy state that fetches data on first use
 * and subscribes to events only when needed
 */
export function useLazyState<C extends SeelenCommand>(
  config: LazyStateConfig<C>,
): Scope<AllSeelenCommandReturns[C]> {
  const { command, event } = config;

  const [scope, setScope] = useState<Scope<AllSeelenCommandReturns[C]>>({
    fetching: true,
    data: null,
  });

  useEffect(() => {
    let unsubscribe: (() => void) | undefined;

    // Subscribe to event if provided
    if (event) {
      const subscribePromise = subscribe(event, ({ payload }) => {
        setScope({ fetching: false, data: payload as any });
      });

      subscribePromise.then((unsub) => {
        unsubscribe = unsub;
      });
    }

    // Fetch initial data
    const fetchData = async () => {
      try {
        const data = await invoke(command as any);
        setScope((prev) => {
          // Only set if not already set by event
          if (prev.fetching) {
            return { fetching: false, data };
          }
          return prev;
        });
      } catch (error) {
        console.error(`Failed to fetch data for command ${command}:`, error);
        setScope({ fetching: false, data: null });
      }
    };

    fetchData();

    return () => {
      if (unsubscribe) {
        unsubscribe();
      }
    };
  }, [command, event]);

  return scope;
}

// Specific hooks for each scope requirement

export function useLazyUser() {
  return useLazyState({
    command: SeelenCommand.GetUser,
    event: SeelenEvent.UserChanged,
  });
}

export function useLazyLanguages() {
  return useLazyState({
    command: SeelenCommand.SystemGetLanguages,
    event: SeelenEvent.SystemLanguagesChanged,
  });
}

export function useLazyPowerStatus() {
  return useLazyState({
    command: SeelenCommand.GetPowerStatus,
    event: SeelenEvent.PowerStatus,
  });
}

export function useLazyPowerMode() {
  return useLazyState({
    command: SeelenCommand.GetPowerMode,
    event: SeelenEvent.PowerMode,
  });
}

export function useLazyBatteries() {
  return useLazyState({
    command: SeelenCommand.GetBatteries,
    event: SeelenEvent.BatteriesStatus,
  });
}

export function useLazyMediaSessions() {
  return useLazyState({
    command: SeelenCommand.GetMediaSessions,
    event: SeelenEvent.MediaSessions,
  });
}

export function useLazyMediaDevices() {
  return useLazyState({
    command: SeelenCommand.GetMediaDevices,
    event: SeelenEvent.MediaDevices,
  });
}

export function useLazyNetworkAdapters() {
  return useLazyState({
    command: SeelenCommand.GetNetworkAdapters,
    event: SeelenEvent.NetworkAdapters,
  });
}

export function useLazyNetworkLocalIp() {
  return useLazyState({
    command: SeelenCommand.GetNetworkDefaultLocalIp,
    event: SeelenEvent.NetworkDefaultLocalIp,
  });
}

export function useLazyOnline() {
  return useLazyState({
    command: SeelenCommand.GetNetworkInternetConnection,
    event: SeelenEvent.NetworkInternetConnection,
  });
}

export function useLazyBluetoothDevices() {
  return useLazyState({
    command: SeelenCommand.GetBluetoothDevices,
    event: SeelenEvent.BluetoothDevicesChanged,
  });
}

export function useLazyNotifications() {
  return useLazyState({
    command: SeelenCommand.GetNotifications,
    event: SeelenEvent.Notifications,
  });
}

export function useLazyDisks() {
  return useLazyState({
    command: SeelenCommand.GetSystemDisks,
    event: SeelenEvent.SystemDisksChanged,
  });
}

export function useLazyNetworkStatistics() {
  return useLazyState({
    command: SeelenCommand.GetSystemNetwork,
    event: SeelenEvent.SystemNetworkChanged,
  });
}

export function useLazyMemory() {
  return useLazyState({
    command: SeelenCommand.GetSystemMemory,
    event: SeelenEvent.SystemMemoryChanged,
  });
}

export function useLazyCores() {
  return useLazyState({
    command: SeelenCommand.GetSystemCores,
    event: SeelenEvent.SystemCoresChanged,
  });
}
