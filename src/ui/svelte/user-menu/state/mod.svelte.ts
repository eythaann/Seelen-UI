import { invoke, SeelenCommand, SeelenEvent, Settings, subscribe, Widget } from "@seelen-ui/lib";
import type { FolderType, User } from "@seelen-ui/lib/types";
import { locale } from "../i18n/index.ts";
import { writable } from "svelte/store";
import { lazyRune } from "libs/ui/svelte/utils/LazyRune.svelte.ts";

const widget = Widget.getCurrent();
const webview = widget.webview;

const settings = writable(await Settings.getAsync());
Settings.onChange((s) => settings.set(s));
settings.subscribe((settings) => {
  locale.set(settings.language || "en");
});

const user = lazyRune(() => invoke(SeelenCommand.GetUser));
subscribe(SeelenEvent.UserChanged, user.setByPayload);
await user.init();

let openCategory = $state<FolderType | null>(null);
webview.onFocusChanged((e) => {
  if (!e.payload) {
    openCategory = null;
  }
});

class State {
  get user(): User {
    return user.value;
  }

  get openCategory(): FolderType | null {
    return openCategory;
  }
  set openCategory(value: FolderType | null) {
    openCategory = value;
  }
}

export const globalState = new State();
