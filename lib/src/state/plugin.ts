import { listen, UnlistenFn } from '@tauri-apps/api/event';

import { invoke, SeelenCommand, SeelenEvent } from '../handlers';
import { getCurrentWidget } from '../utils';

export class Plugin {
  id: string = '';
  target: string = '';
  plugin: any = {};
}

export class PluginList {
  private constructor(private inner: Plugin[]) {}

  static async getAsync(): Promise<PluginList> {
    return new PluginList(await invoke(SeelenCommand.StateGetPlugins));
  }

  static async onChange(cb: (value: PluginList) => void): Promise<UnlistenFn> {
    return listen<Plugin[]>(SeelenEvent.StatePluginsChanged, (event) => {
      cb(new PluginList(event.payload));
    });
  }

  all(): Plugin[] {
    return this.inner;
  }

  forCurrentWidget(): Plugin[] {
    let target = getCurrentWidget().id;
    return this.inner.filter((plugin) => plugin.target === target);
  }
}
