import { invoke, SeelenCommand, SeelenEvent, Settings, subscribe, Widget } from "@seelen-ui/lib";
import { RadioDeviceKind, type WlanBssEntry } from "@seelen-ui/lib/types";
import { locale } from "./i18n/index.ts";
import { writable } from "svelte/store";
import { lazyRune } from "libs/ui/svelte/utils/LazyRune.svelte.ts";

let widget = Widget.getCurrent();
let webview = widget.webview;

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
await subscribe(SeelenEvent.RadiosChanged, radios.setByPayload);
await radios.init();

let isScanning = $state(false);
let selectedSsid = $state<string | null>(null);

webview.onFocusChanged(async (e) => {
  if (e.payload) {
    await invoke(SeelenCommand.WlanStartScanning);
    isScanning = true;
  } else {
    await invoke(SeelenCommand.WlanStopScanning);
    isScanning = false;
    selectedSsid = null;
  }
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
