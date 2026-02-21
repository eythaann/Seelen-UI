import { invoke, SeelenCommand, SeelenEvent, subscribe } from "@seelen-ui/lib";
import { lazyRune } from "libs/ui/svelte/utils";

let mediaDevices = lazyRune(() => invoke(SeelenCommand.GetMediaDevices));
subscribe(SeelenEvent.MediaDevices, mediaDevices.setByPayload);

let mediaPlaying = lazyRune(() => invoke(SeelenCommand.GetMediaSessions));
subscribe(SeelenEvent.MediaSessions, mediaPlaying.setByPayload);

let brightness = lazyRune(() => invoke(SeelenCommand.GetAllMonitorsBrightness));
subscribe(SeelenEvent.SystemMonitorsBrightnessChanged, brightness.setByPayload);

let workspaces = lazyRune(() => invoke(SeelenCommand.StateGetVirtualDesktops));
subscribe(SeelenEvent.VirtualDesktopsChanged, workspaces.setByPayload);

let notifications = lazyRune(() => invoke(SeelenCommand.GetNotifications));
subscribe(SeelenEvent.Notifications, notifications.setByPayload);

await Promise.all([
  mediaDevices.init(),
  mediaPlaying.init(),
  brightness.init(),
  workspaces.init(),
  notifications.init(),
]);

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
    return brightness.value[0] || null;
  },
  get workspaces() {
    return workspaces.value;
  },
  get notifications() {
    return notifications.value;
  },
};
