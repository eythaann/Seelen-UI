import { SeelenCommand, SeelenEvent, type UnSubscriber } from "../handlers/mod.ts";
import { List } from "../utils/List.ts";
import { newFromInvoke, newOnEvent } from "../utils/State.ts";

export interface KeyboardLayout {
  id: string;
  handle: string;
  displayName: string;
  active: boolean;
}

export interface SystemLanguage {
  id: string;
  code: string;
  name: string;
  nativeName: string;
  inputMethods: KeyboardLayout[];
}

export class LanguageList extends List<SystemLanguage> {
  static getAsync(): Promise<LanguageList> {
    return newFromInvoke(this, SeelenCommand.SystemGetLanguages);
  }

  static onChange(cb: (payload: LanguageList) => void): Promise<UnSubscriber> {
    return newOnEvent(cb, this, SeelenEvent.SystemLanguagesChanged);
  }
}
