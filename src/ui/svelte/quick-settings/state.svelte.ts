import { invoke, SeelenCommand, SeelenEvent, subscribe } from "@seelen-ui/lib";
import type { Brightness, MediaDevice, RadioDevice } from "@seelen-ui/lib/types";
import { lazyRune } from "libs/ui/svelte/utils";

// Initialize lazy signals
const brightness = lazyRune(() => {
  return invoke(SeelenCommand.GetMainMonitorBrightness);
});
await brightness.init();

const mediaDevices = lazyRune(async () => {
  const [inputs, outputs] = await invoke(SeelenCommand.GetMediaDevices);
  return { inputs, outputs };
});
await subscribe(SeelenEvent.MediaDevices, ({ payload: [inputs, outputs] }) => {
  mediaDevices.value = { inputs, outputs };
});
await mediaDevices.init();

const radios = lazyRune(() => invoke(SeelenCommand.GetRadios));
await subscribe(SeelenEvent.RadiosChanged, radios.setByPayload);
await radios.init();

class State {
  get brightness(): Brightness | null {
    return brightness.value;
  }
  set brightness(value: Brightness | null) {
    brightness.value = value;
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
