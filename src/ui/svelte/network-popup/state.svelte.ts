import { invoke, SeelenCommand, SeelenEvent, Settings, subscribe, Widget } from "@seelen-ui/lib";
import {
  AdapterStatus,
  type Hotspot,
  type NetworkAdapter,
  RadioDeviceKind,
  type WlanBssEntry,
} from "@seelen-ui/lib/types";
import { locale } from "./i18n/index.ts";
import { lazyRune } from "libs/ui/svelte/utils/LazyRune.svelte.ts";

let widget = Widget.getCurrent();

const settings = lazyRune(() => Settings.getAsync());
Settings.onChange((s) => (settings.value = s));
await settings.init();

const isDevtoolsEnabled = $derived(settings.value.devTools);

$effect.root(() => {
  $effect(() => {
    locale.set(settings.value.language);
  });
});

let wlanBssEntries = $state<WlanBssEntry[]>([]);
subscribe(SeelenEvent.NetworkWlanScanned, ({ payload }) => {
  wlanBssEntries = payload;
});

let radios = lazyRune(() => invoke(SeelenCommand.GetRadios));
subscribe(SeelenEvent.RadiosChanged, radios.setByPayload);
await radios.init();

let hotspot = lazyRune(() => invoke(SeelenCommand.GetNetworkHotspot));
subscribe(SeelenEvent.NetworkHotspotChanged, hotspot.setByPayload);
await hotspot.init();

let adapters = lazyRune(() => invoke(SeelenCommand.GetNetworkAdapters));
subscribe(SeelenEvent.NetworkAdapters, adapters.setByPayload);
await adapters.init();

let online = lazyRune(() => invoke(SeelenCommand.GetNetworkInternetConnection));
subscribe(SeelenEvent.NetworkInternetConnection, online.setByPayload);
await online.init();

/**
 * Ethernet interfaces with link, physical or virtual. Unplugged ones are hidden (they would
 * only add noise on laptops/docks with an unused port). The one holding the default route
 * goes first.
 */
const ethernetAdapters = $derived(
  adapters.value
    .filter((a) => /* a.type === "ETHERNET_CSMACD" && */ a.status === AdapterStatus.up)
    .toSorted((a, b) => Number(b.isDefault) - Number(a.isDefault)),
);

// Scans while the popup is visible. Visibility (not focus) is the signal: a popup opened by hover
// is never focused, and the focus event can be missed when the window is created by the trigger.
let isScanning = $state(widget.visible);
let selectedSsid = $state<string | null>(null);
let currentView = $state<"main" | "hotspot">("main");

widget.onVisibilityChange((visible) => {
  isScanning = visible;
  if (!visible) {
    selectedSsid = null;
    currentView = "main";
  }
});

$effect.root(() => {
  $effect(() => {
    const wifiEnabled = radios.value.some(
      (radio) => radio.kind === RadioDeviceKind.WiFi && radio.isEnabled,
    );
    if (!isScanning || !wifiEnabled) {
      return;
    }

    // the backend reports the networks already known by the OS right away, then after the scan
    invoke(SeelenCommand.WlanScan);
    const interval = setInterval(() => invoke(SeelenCommand.WlanScan), 2000);
    return () => clearInterval(interval);
  });
});

class State {
  get ethernetAdapters(): NetworkAdapter[] {
    return ethernetAdapters;
  }

  get online(): boolean {
    return online.value;
  }

  get wifiRadio() {
    return radios.value.find((radio) => radio.kind === RadioDeviceKind.WiFi);
  }

  get wlanBssEntries() {
    return wlanBssEntries;
  }

  get hotspot(): Hotspot | null {
    return hotspot.value;
  }

  get view() {
    return currentView;
  }

  set view(value: "main" | "hotspot") {
    currentView = value;
  }

  get isScanning() {
    return isScanning;
  }

  set isScanning(value: boolean) {
    isScanning = value;
  }

  get selectedSsid() {
    return selectedSsid;
  }

  set selectedSsid(value: string | null) {
    selectedSsid = value;
  }

  get isDevtoolsEnabled() {
    return isDevtoolsEnabled;
  }
}

export const globalState = new State();
