import { SeelenCommand, SeelenEvent, type UnSubscriber } from "../../handlers/mod.ts";

import type { Settings as ISettings, ThirdPartyWidgetSettings } from "@seelen-ui/types";
import { newFromInvoke, newOnEvent } from "../../utils/State.ts";
import { invoke } from "../../handlers/mod.ts";
import { Widget } from "../widget/mod.ts";

export interface Settings extends ISettings {}
export class Settings {
  constructor(public inner: ISettings) {
    Object.assign(this, this.inner);
  }

  static default(): Promise<Settings> {
    return newFromInvoke(this, SeelenCommand.StateGetDefaultSettings);
  }

  static getAsync(): Promise<Settings> {
    return newFromInvoke(this, SeelenCommand.StateGetSettings);
  }

  static onChange(cb: (payload: Settings) => void): Promise<UnSubscriber> {
    return newOnEvent(cb, this, SeelenEvent.StateSettingsChanged);
  }

  static loadCustom(path: string): Promise<Settings> {
    return newFromInvoke(this, SeelenCommand.StateGetSettings, { path });
  }

  /**
   * Returns the settings for the current widget, taking in care of the replicas
   * the returned object will be a merge of:
   * - the default settings set on the widget definition
   * - the stored user settings
   * - the instance patch settings (if apply)
   * - the monitor patch settings (if apply)
   */
  getCurrentWidgetConfig(): ThirdPartyWidgetSettings {
    const currentWidget = Widget.getCurrent();

    const widgetId = currentWidget.id;
    const { monitorId, instanceId } = currentWidget.decoded;

    const root = this.inner.byWidget[widgetId];
    const instance = instanceId ? root?.$instances?.[instanceId] : undefined;
    const monitor = monitorId ? this.inner.monitorsV3[monitorId]?.byWidget[widgetId] : undefined;

    return {
      ...currentWidget.getDefaultConfig(),
      ...(root || {}),
      ...(instance || {}),
      ...(monitor || {}),
    };
  }

  /** Will store the settings on disk */
  save(): Promise<void> {
    return invoke(SeelenCommand.StateWriteSettings, { settings: this.inner });
  }
}

export * from "./settings_by_monitor.ts";
