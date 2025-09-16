import { SeelenCommand, SeelenEvent, type UnSubscriber } from "../handlers/mod.ts";
import { newFromInvoke, newOnEvent } from "../utils/State.ts";
import type { LauncherHistory as ILauncherHistory } from "@seelen-ui/types";

export * from "./theme/mod.ts";
export * from "./settings/mod.ts";
export * from "./weg_items.ts";
export * from "./wm_layout.ts";
export * from "./placeholder.ts";
export * from "./settings_by_app.ts";
export * from "./settings/settings_by_monitor.ts";
export * from "./icon_pack.ts";
export * from "./plugin/mod.ts";
export * from "./widget/mod.ts";
export * from "./profile.ts";
export * from "./wallpaper/mod.ts";
export * from "./startup.ts";

export class LauncherHistory {
  constructor(public inner: ILauncherHistory) {}

  static getAsync(): Promise<LauncherHistory> {
    return newFromInvoke(this, SeelenCommand.StateGetHistory);
  }

  static onChange(
    cb: (payload: LauncherHistory) => void,
  ): Promise<UnSubscriber> {
    return newOnEvent(cb, this, SeelenEvent.StateHistoryChanged);
  }
}
