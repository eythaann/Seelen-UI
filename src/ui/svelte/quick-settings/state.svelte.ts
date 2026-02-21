import { invoke, SeelenCommand, SeelenEvent, subscribe } from "@seelen-ui/lib";
import type { MediaDevice, RadioDevice } from "@seelen-ui/lib/types";
import { lazyRune } from "libs/ui/svelte/utils";

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

await Promise.all([brightness.init(), mediaDevices.init(), radios.init()]);

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
}
export const state = new State();
