import { invoke, SeelenCommand, SeelenEvent, subscribe } from "@seelen-ui/lib";
import { lazyRune } from "libs/ui/svelte/utils";

const langs = lazyRune(() => invoke(SeelenCommand.SystemGetLanguages));
subscribe(SeelenEvent.SystemLanguagesChanged, langs.setByPayload);
await langs.init();

export const state = {
  get langs() {
    return langs.value;
  },
};
