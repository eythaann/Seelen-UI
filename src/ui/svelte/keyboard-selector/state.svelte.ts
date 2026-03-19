import { invoke, SeelenCommand, SeelenEvent, subscribe } from "@seelen-ui/lib";
import { lazyRune } from "libs/ui/svelte/utils";
import { locale } from "./i18n/index.ts";

let settings = lazyRune(() => invoke(SeelenCommand.StateGetSettings, { path: null }));
subscribe(SeelenEvent.StateSettingsChanged, settings.setByPayload);
await settings.init();
$effect.root(() => {
  $effect(() => {
    locale.set(settings.value.language || "en");
  });
});

const langs = lazyRune(() => invoke(SeelenCommand.SystemGetLanguages));
subscribe(SeelenEvent.SystemLanguagesChanged, langs.setByPayload);
await langs.init();

export const state = {
  get langs() {
    return langs.value;
  },
};
