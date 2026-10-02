import { invoke, SeelenCommand, SeelenEvent, Settings, subscribe, Widget } from "@seelen-ui/lib";
import { locale } from "./i18n/index.ts";
import { lazyRune } from "libs/ui/svelte/utils/LazyRune.svelte.ts";
import { rootEffectAsync } from "libs/ui/svelte/utils/RootEffect.svelte.ts";
import { monitors } from "libs/ui/svelte/runes/monitors.svelte.ts";

const settings = lazyRune(() => Settings.getAsync());
Settings.onChange((s) => (settings.value = s));
await settings.init();

$effect.root(() => {
  $effect(() => {
    locale.set(settings.value.language);
  });
});

let user = lazyRune(() => invoke(SeelenCommand.GetUser));
subscribe(SeelenEvent.UserChanged, user.setByPayload);

let hasPendingOsUpdates = $state(false);
const refreshPendingOsUpdates = async () => {
  hasPendingOsUpdates = await invoke(SeelenCommand.HasPendingOsUpdates);
};

await Promise.all([user.init(), monitors.init(), refreshPendingOsUpdates()]);

const posSet = rootEffectAsync(() => Widget.self.setPosition(monitors.desktopRect));

Widget.self.onTrigger(async () => {
  invoke(SeelenCommand.GetUser); // refresh user information
  refreshPendingOsUpdates();
  await posSet; // wait for the initial position to be set
  await Widget.self.show();
  await Widget.self.focus();
});

export type State = typeof state;
export const state = {
  get user() {
    return user.value;
  },
  get hasPendingOsUpdates() {
    return hasPendingOsUpdates;
  },
};
