import type { AppConfig, AppExtraFlag, AppIdentifierType, MatchingStrategy } from "@seelen-ui/types";
import { List } from "../utils/List.ts";
import { SeelenCommand, SeelenEvent, type UnSubscriber } from "../handlers/mod.ts";
import { newFromInvoke } from "../utils/State.ts";
import { newOnEvent } from "../utils/State.ts";
import type { Enum } from "../utils/enums.ts";

export class AppConfigurationList extends List<AppConfig> {
  static getAsync(): Promise<AppConfigurationList> {
    return newFromInvoke(
      this,
      SeelenCommand.StateGetSpecificAppsConfigurations,
    );
  }

  static onChange(
    cb: (payload: AppConfigurationList) => void,
  ): Promise<UnSubscriber> {
    return newOnEvent(cb, this, SeelenEvent.StateSettingsByAppChanged);
  }
}

// =================================================================================
//    From here some enums as helpers like @seelen-ui/types only contains types
// =================================================================================

const AppExtraFlag: Enum<AppExtraFlag> = {
  Float: "float",
  Force: "force",
  Unmanage: "unmanage",
  Pinned: "pinned",
};

const AppIdentifierType: Enum<AppIdentifierType> = {
  Exe: "Exe",
  Class: "Class",
  Title: "Title",
  Path: "Path",
};

const MatchingStrategy: Enum<MatchingStrategy> = {
  Equals: "Equals",
  StartsWith: "StartsWith",
  EndsWith: "EndsWith",
  Contains: "Contains",
  Regex: "Regex",
};

export { AppExtraFlag, AppIdentifierType, MatchingStrategy };
