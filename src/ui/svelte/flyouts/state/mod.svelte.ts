import { invoke, SeelenCommand, SeelenEvent, subscribe } from "@seelen-ui/lib";
import { lazyRune } from "libs/ui/svelte/utils";

let mediaDevices = lazyRune(() => invoke(SeelenCommand.GetMediaDevices));
await subscribe(SeelenEvent.MediaDevices, mediaDevices.setByPayload);
await mediaDevices.init();

let mediaPlaying = lazyRune(() => invoke(SeelenCommand.GetMediaSessions));
await subscribe(SeelenEvent.MediaSessions, mediaPlaying.setByPayload);
await mediaPlaying.init();

let brightness = lazyRune(() => invoke(SeelenCommand.GetMainMonitorBrightness));
await subscribe(SeelenEvent.SystemBrightnessChanged, brightness.setByPayload);
await brightness.init();

let workspaces = lazyRune(() => invoke(SeelenCommand.StateGetVirtualDesktops));
await subscribe(SeelenEvent.VirtualDesktopsChanged, workspaces.setByPayload);
await workspaces.init();

export const state = {
  get mediaInputs() {
    return mediaDevices.value[0];
  },
  get mediaOutputs() {
    return mediaDevices.value[1];
  },
  get mediaPlaying() {
    return mediaPlaying.value;
  },
  get brightness() {
    return brightness.value;
  },
  get workspaces() {
    return workspaces.value;
  },
};
