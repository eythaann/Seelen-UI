import type { Profile } from "@seelen-ui/types";
import { SeelenCommand } from "../handlers/mod.ts";
import { List } from "../utils/List.ts";
import { newFromInvoke } from "../utils/State.ts";

export class ProfileList extends List<Profile> {
  static getAsync(): Promise<ProfileList> {
    return newFromInvoke(this, SeelenCommand.StateGetProfiles);
  }
}
