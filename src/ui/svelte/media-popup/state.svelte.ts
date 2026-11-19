import { invoke, SeelenCommand, SeelenEvent, Settings, subscribe, Widget } from "@seelen-ui/lib";
import type { MediaDevice, MediaPlayer } from "@seelen-ui/lib/types";
import { locale } from "./i18n/index.ts";
import { lazyRune } from "libs/ui/svelte/utils/LazyRune.svelte.ts";
import { DEFAULT_VIEW, type MediaPopupView, resolveLastView, serializeLastView } from "./lastView.ts";

let widget = Widget.getCurrent();

const settings = lazyRune(() => Settings.getAsync());
Settings.onChange((s) => (settings.value = s));
await settings.init();

$effect.root(() => {
  $effect(() => {
    locale.set(settings.value.language);
  });
});

let mediaDevices = lazyRune(() => invoke(SeelenCommand.GetMediaDevices));
subscribe(SeelenEvent.MediaDevices, mediaDevices.setByPayload);

let mediaSessions = lazyRune(() => invoke(SeelenCommand.GetMediaSessions));
subscribe(SeelenEvent.MediaSessions, mediaSessions.setByPayload);

await Promise.all([mediaDevices.init(), mediaSessions.init()]);

const LAST_VIEW_STORAGE_KEY = "lastView";

/** Widget setting `rememberLastView`: reopen on the last view instead of the device overview. */
function isRememberLastViewEnabled(): boolean {
  return settings.value.getCurrentWidgetConfig().rememberLastView === true;
}

function readStoredLastView(): string | null {
  try {
    return localStorage.getItem(LAST_VIEW_STORAGE_KEY);
  } catch {
    return null;
  }
}

const initialView = isRememberLastViewEnabled()
  ? resolveLastView(
    readStoredLastView(),
    [...mediaDevices.value[0], ...mediaDevices.value[1]].map((d) => d.id),
  )
  : DEFAULT_VIEW;

let currentView = $state<MediaPopupView>(initialView.view);
let selectedDeviceId = $state<string | null>(initialView.deviceId);

function storeLastView(): void {
  if (!isRememberLastViewEnabled()) return;
  try {
    localStorage.setItem(
      LAST_VIEW_STORAGE_KEY,
      serializeLastView({ view: currentView, deviceId: selectedDeviceId }),
    );
  } catch (e) {
    console.error("Failed to store media popup last view", e);
  }
}

widget.window.onFocusChanged((e) => {
  // Reset state when popup loses focus, unless the user wants the last view remembered
  if (!e.payload && !isRememberLastViewEnabled()) {
    currentView = "main";
    selectedDeviceId = null;
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

  set view(value: MediaPopupView) {
    currentView = value;
    storeLastView();
  }

  get selectedDeviceId() {
    return selectedDeviceId;
  }

  set selectedDeviceId(value: string | null) {
    selectedDeviceId = value;
    storeLastView();
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
