import { invoke, SeelenCommand, SeelenEvent, Settings, subscribe, Widget } from "@seelen-ui/lib";
import { RadioDeviceKind, type WlanBssEntry } from "@seelen-ui/lib/types";
import { locale } from "./i18n/index.ts";
import { writable } from "svelte/store";
import { lazyRune } from "libs/ui/svelte/utils/LazyRune.svelte.ts";

let widget = Widget.getCurrent();

let settings = writable(await Settings.getAsync());
Settings.onChange((s) => settings.set(s));
settings.subscribe((settings) => {
  locale.set(settings.language || "en");
});

let wlanBssEntries = $state<WlanBssEntry[]>([]);
subscribe(SeelenEvent.NetworkWlanScanned, ({ payload }) => {
  wlanBssEntries = payload;
});

let radios = lazyRune(() => invoke(SeelenCommand.GetRadios));
subscribe(SeelenEvent.RadiosChanged, radios.setByPayload);
await radios.init();

let isScanning = $state(false);
let selectedSsid = $state<string | null>(null);
let scanInterval: ReturnType<typeof setInterval> | null = null;

widget.window.onFocusChanged((e) => {
  if (e.payload) {
    isScanning = true;
  } else {
    isScanning = false;
    selectedSsid = null;
  }
});

$effect.root(() => {
  $effect(() => {
    if (isScanning) {
      invoke(SeelenCommand.WlanScan);
      scanInterval = setInterval(() => invoke(SeelenCommand.WlanScan), 2000);
      return;
    }

    if (scanInterval !== null) {
      clearInterval(scanInterval);
      scanInterval = null;
    }
  });
});

class State {
  get wifiRadio() {
    return radios.value.find((radio) => radio.kind === RadioDeviceKind.WiFi);
  }

  get wlanBssEntries() {
    return wlanBssEntries;
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
}

export const globalState = new State();
