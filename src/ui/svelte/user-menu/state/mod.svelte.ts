import { invoke, SeelenCommand, SeelenEvent, Settings, subscribe, Widget } from "@seelen-ui/lib";
import type { FolderType, User } from "@seelen-ui/lib/types";
import { locale } from "../i18n/index.ts";
import { lazyRune } from "libs/ui/svelte/utils/LazyRune.svelte.ts";

const widget = Widget.getCurrent();

const settings = lazyRune(() => Settings.getAsync());
Settings.onChange((s) => (settings.value = s));
await settings.init();

$effect.root(() => {
  $effect(() => {
    locale.set(settings.value.language || "en");
  });
});

const user = lazyRune(() => invoke(SeelenCommand.GetUser));
subscribe(SeelenEvent.UserChanged, user.setByPayload);
await user.init();

let openCategory = $state<FolderType | null>(null);
widget.window.onFocusChanged((e) => {
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
