import { SeelenCommand, SeelenEvent, type UnSubscriber } from "../handlers/mod.ts";
import { List } from "../utils/List.ts";
import { newFromInvoke, newOnEvent } from "../utils/State.ts";
import type { SystemLanguage } from "@seelen-ui/types";

export class LanguageList extends List<SystemLanguage> {
  static getAsync(): Promise<LanguageList> {
    return newFromInvoke(this, SeelenCommand.SystemGetLanguages);
  }

  static onChange(cb: (payload: LanguageList) => void): Promise<UnSubscriber> {
    return newOnEvent(cb, this, SeelenEvent.SystemLanguagesChanged);
  }
}
