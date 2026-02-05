import { lazySignal } from "libs/ui/react/utils/LazySignal";
import { invoke, SeelenCommand, SeelenEvent, subscribe } from "@seelen-ui/lib";
import type { WlanBssEntry } from "@seelen-ui/lib/types";

// User and Environment Data
export const $env = lazySignal(() => invoke(SeelenCommand.GetUserEnvs));

export const $user = lazySignal(() => invoke(SeelenCommand.GetUser));
subscribe(SeelenEvent.UserChanged, $user.setByPayload);

export const $languages = lazySignal(() => invoke(SeelenCommand.SystemGetLanguages));
subscribe(SeelenEvent.SystemLanguagesChanged, $languages.setByPayload);

// Power and Battery
export const $power_status = lazySignal(() => invoke(SeelenCommand.GetPowerStatus));
subscribe(SeelenEvent.PowerStatus, $power_status.setByPayload);

export const $power_plan = lazySignal(() => invoke(SeelenCommand.GetPowerMode));
subscribe(SeelenEvent.PowerMode, $power_plan.setByPayload);

export const $batteries = lazySignal(() => invoke(SeelenCommand.GetBatteries));
subscribe(SeelenEvent.BatteriesStatus, $batteries.setByPayload);

// Media
export const $media_sessions = lazySignal(() => invoke(SeelenCommand.GetMediaSessions));
subscribe(SeelenEvent.MediaSessions, $media_sessions.setByPayload);

export const $media_outputs = lazySignal(async () => {
  const [, outputs] = await invoke(SeelenCommand.GetMediaDevices);
  return outputs;
});
subscribe(SeelenEvent.MediaOutputs, $media_outputs.setByPayload);

export const $media_inputs = lazySignal(async () => {
  const [inputs] = await invoke(SeelenCommand.GetMediaDevices);
  return inputs;
});
subscribe(SeelenEvent.MediaInputs, $media_inputs.setByPayload);

// Network
export const $network_adapters = lazySignal(() => invoke(SeelenCommand.GetNetworkAdapters));
subscribe(SeelenEvent.NetworkAdapters, $network_adapters.setByPayload);

export const $network_local_ip = lazySignal(() => invoke(SeelenCommand.GetNetworkDefaultLocalIp));
subscribe(SeelenEvent.NetworkDefaultLocalIp, $network_local_ip.setByPayload);

export const $online = lazySignal(() => invoke(SeelenCommand.GetNetworkInternetConnection));
(SeelenEvent.NetworkInternetConnection, $online.setByPayload);

export const $wlan_bss_entries = lazySignal<WlanBssEntry[]>(() => Promise.resolve([]));
subscribe(SeelenEvent.NetworkWlanScanned, $wlan_bss_entries.setByPayload);

// Bluetooth
export const $bluetooth_devices = lazySignal(() => invoke(SeelenCommand.GetBluetoothDevices));
subscribe(SeelenEvent.BluetoothDevicesChanged, $bluetooth_devices.setByPayload);

// Notifications
export const $notifications = lazySignal(() => invoke(SeelenCommand.GetNotifications));
subscribe(SeelenEvent.Notifications, $notifications.setByPayload);

export const $disks = lazySignal(() => invoke(SeelenCommand.GetSystemDisks));
subscribe(SeelenEvent.SystemDisksChanged, $disks.setByPayload);

export const $network_statistics = lazySignal(() => invoke(SeelenCommand.GetSystemNetwork));
subscribe(SeelenEvent.SystemNetworkChanged, $network_statistics.setByPayload);

export const $memory = lazySignal(() => invoke(SeelenCommand.GetSystemMemory));
subscribe(SeelenEvent.SystemMemoryChanged, $memory.setByPayload);

export const $cores = lazySignal(() => invoke(SeelenCommand.GetSystemCores));
subscribe(SeelenEvent.SystemCoresChanged, $cores.setByPayload);

await Promise.all([
  $user.init(),
  $env.init(),
  $languages.init(),
  $power_status.init(),
  $power_plan.init(),
  $batteries.init(),
  $media_sessions.init(),
  $media_outputs.init(),
  $media_inputs.init(),
  $network_adapters.init(),
  $network_local_ip.init(),
  $online.init(),
  $wlan_bss_entries.init(),
  $bluetooth_devices.init(),
  $notifications.init(),
  $disks.init(),
  $network_statistics.init(),
  $memory.init(),
  $cores.init(),
]);
