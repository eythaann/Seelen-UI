import { invoke, SeelenCommand, SeelenEvent, Settings, subscribe, Widget } from "@seelen-ui/lib";
import type { MediaDevice, MediaPlayer } from "@seelen-ui/lib/types";
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

let mediaDevices = lazyRune(() => invoke(SeelenCommand.GetMediaDevices));
await subscribe(SeelenEvent.MediaDevices, mediaDevices.setByPayload);
await mediaDevices.init();

let mediaSessions = lazyRune(() => invoke(SeelenCommand.GetMediaSessions));
await subscribe(SeelenEvent.MediaSessions, mediaSessions.setByPayload);
await mediaSessions.init();

let currentView = $state<"main" | "mixer">("main");
let selectedDeviceId = $state<string | null>(null);

webview.onFocusChanged((e) => {
  if (!e.payload) {
    // Reset state when popup loses focus
    currentView = "main";
    selectedDeviceId = null;
    webview.hide();
  }
});

class State {
  get inputs(): MediaDevice[] {
    return mediaDevices.value[0];
  }

  get outputs(): MediaDevice[] {
    return mediaDevices.value[1];
  }

  get sessions(): MediaPlayer[] {
    return mediaSessions.value;
  }

  get defaultInput(): MediaDevice | undefined {
    return this.inputs.find((d) => d.isDefaultMultimedia);
  }

  get defaultOutput(): MediaDevice | undefined {
    return this.outputs.find((d) => d.isDefaultMultimedia);
  }

  get view() {
    return currentView;
  }

  set view(value: "main" | "mixer") {
    currentView = value;
  }

  get selectedDeviceId() {
    return selectedDeviceId;
  }

  set selectedDeviceId(value: string | null) {
    selectedDeviceId = value;
  }

  get selectedDevice(): MediaDevice | undefined {
    if (!selectedDeviceId) return undefined;
    return (
      this.inputs.find((d) => d.id === selectedDeviceId) ||
      this.outputs.find((d) => d.id === selectedDeviceId)
    );
  }
}

export const globalState = new State();
