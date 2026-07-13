import { invoke, SeelenCommand, SeelenEvent, Settings, subscribe, Widget } from "@seelen-ui/lib";
import { RadioDeviceKind } from "@seelen-ui/lib/types";
import { locale } from "./i18n/index.ts";
import { lazyRune } from "libs/ui/svelte/utils/LazyRune.svelte.ts";

let widget = Widget.getCurrent();

const settings = lazyRune(() => Settings.getAsync());
Settings.onChange((s) => (settings.value = s));
await settings.init();

$effect.root(() => {
  $effect(() => {
    locale.set(settings.value.language);
  });
});

let devices = lazyRune(() => invoke(SeelenCommand.GetBluetoothDevices));
subscribe(SeelenEvent.BluetoothDevicesChanged, devices.setByPayload);

let radios = lazyRune(() => invoke(SeelenCommand.GetRadios));
subscribe(SeelenEvent.RadiosChanged, radios.setByPayload);

await Promise.all([devices.init(), radios.init()]);

export enum BluetoothOperation {
  Connecting = "connecting",
  Disconnecting = "disconnecting",
  Pairing = "pairing",
  Unpairing = "unpairing",
}

let isScanning = $state(false);
let selectedDeviceId = $state<string | null>(null);
let loadingDeviceId = $state<string | null>(null);
let loadingOperation = $state<BluetoothOperation | null>(null);

widget.window.onFocusChanged(async (e) => {
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

  get loadingDeviceId() {
    return loadingDeviceId;
  }
  set loadingDeviceId(value: string | null) {
    loadingDeviceId = value;
  }

  get loadingOperation() {
    return loadingOperation;
  }
  set loadingOperation(value: BluetoothOperation | null) {
    loadingOperation = value;
  }
}

export const globalState = new State();
