import type { AppConfig } from "@seelen-ui/types";
import { List } from "../utils/List.ts";
import { SeelenCommand, SeelenEvent, type UnSubscriber } from "../handlers/mod.ts";
import { newFromInvoke } from "../utils/State.ts";
import { newOnEvent } from "../utils/State.ts";

export class AppConfigurationList extends List<AppConfig> {
  static getAsync(): Promise<AppConfigurationList> {
    return newFromInvoke(this, SeelenCommand.StateGetSpecificAppsConfigurations);
  }

  static onChange(cb: (payload: AppConfigurationList) => void): Promise<UnSubscriber> {
    return newOnEvent(cb, this, SeelenEvent.StateSettingsByAppChanged);
  }
}
