import { invoke, SeelenCommand, SeelenEvent, Settings, subscribe } from "@seelen-ui/lib";
import type { Hotspot, MediaDevice, PhysicalMonitor, RadioDevice } from "@seelen-ui/lib/types";
import { lazyRune } from "libs/ui/svelte/utils";
import { locale } from "./i18n/index.ts";

const settings = lazyRune(() => Settings.getAsync());
Settings.onChange((s) => (settings.value = s));
await settings.init();

async function setInitialLocale(language: string) {
  try {
    await locale.set(language);
  } catch (error) {
    console.error(`Failed to load initial locale "${language}"`, error);
    if (language === "en") return;

    try {
      await locale.set("en");
    } catch (fallbackError) {
      // Locale loading must never prevent quick settings from mounting.
      console.error('Failed to load fallback locale "en"', fallbackError);
    }
  }
}

await setInitialLocale(settings.value.language);

$effect.root(() => {
  $effect(() => {
    locale.set(settings.value.language).catch((error) => {
      console.error(`Failed to change locale to "${settings.value.language}"`, error);
    });
  });
});

// Initialize lazy signals
const brightness = lazyRune(() => invoke(SeelenCommand.GetAllMonitorsBrightness));
subscribe(SeelenEvent.SystemMonitorsBrightnessChanged, brightness.setByPayload);

const mediaDevices = lazyRune(async () => {
  const [inputs, outputs] = await invoke(SeelenCommand.GetMediaDevices);
  return { inputs, outputs };
});
subscribe(SeelenEvent.MediaDevices, ({ payload: [inputs, outputs] }) => {
  mediaDevices.value = { inputs, outputs };
});

const radios = lazyRune(() => invoke(SeelenCommand.GetRadios));
subscribe(SeelenEvent.RadiosChanged, radios.setByPayload);

const hotspot = lazyRune(() => invoke(SeelenCommand.GetNetworkHotspot));
subscribe(SeelenEvent.NetworkHotspotChanged, hotspot.setByPayload);

const monitors = lazyRune(() => invoke(SeelenCommand.SystemGetMonitors));
subscribe(SeelenEvent.SystemMonitorsChanged, monitors.setByPayload);

const darkMode = lazyRune(() => invoke(SeelenCommand.SystemGetDarkMode));
subscribe(SeelenEvent.DarkModeChanged, darkMode.setByPayload);

const nightLightEnabled = lazyRune(() => invoke(SeelenCommand.SystemGetNightLightEnabled));

await Promise.all([
  brightness.init(),
  mediaDevices.init(),
  radios.init(),
  hotspot.init(),
  monitors.init(),
  darkMode.init(),
  nightLightEnabled.init(),
]);

class State {
  get brightness() {
    return brightness.value;
  }

  get mediaInputs(): MediaDevice[] {
    return mediaDevices.value.inputs;
  }
  get mediaOutputs(): MediaDevice[] {
    return mediaDevices.value.outputs;
  }
  get radios(): RadioDevice[] {
    return radios.value;
  }
  get hotspot(): Hotspot | null {
    return hotspot.value;
  }
  get monitors(): PhysicalMonitor[] {
    return monitors.value;
  }
  get darkMode(): boolean {
    return darkMode.value;
  }
  get nightLightEnabled(): boolean {
    return nightLightEnabled.value;
  }
  set nightLightEnabled(value: boolean) {
    nightLightEnabled.value = value;
  }
}
export const state = new State();
