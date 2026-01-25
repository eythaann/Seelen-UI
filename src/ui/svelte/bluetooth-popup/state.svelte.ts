import { invoke, SeelenCommand, SeelenEvent, Settings, subscribe, Widget } from "@seelen-ui/lib";
import { RadioDeviceKind } from "@seelen-ui/lib/types";
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

let devices = lazyRune(() => invoke(SeelenCommand.GetBluetoothDevices));
await subscribe(SeelenEvent.BluetoothDevicesChanged, devices.setByPayload);
await devices.init();

let radios = lazyRune(() => invoke(SeelenCommand.GetRadios));
await subscribe(SeelenEvent.RadiosChanged, radios.setByPayload);
await radios.init();

let isScanning = $state(false);
let selectedDeviceId = $state<string | null>(null);

webview.onFocusChanged(async (e) => {
  if (!e.payload) {
    await invoke(SeelenCommand.StartBluetoothScanning);
    isScanning = true;
  } else {
    await invoke(SeelenCommand.StopBluetoothScanning);
    isScanning = false;
    selectedDeviceId = null;
  }
});

class State {
  get bluetoothRadio() {
    return radios.value.find((radio) => radio.kind === RadioDeviceKind.Bluetooth);
  }

  get devices() {
    return devices.value;
  }

  get isScanning() {
    return isScanning;
  }
  set isScanning(value: boolean) {
    isScanning = value;
  }

  get selectedDeviceId() {
    return selectedDeviceId;
  }
  set selectedDeviceId(value: string | null) {
    selectedDeviceId = value;
  }
}

export const globalState = new State();
