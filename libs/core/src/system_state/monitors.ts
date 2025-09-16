import { SeelenCommand, SeelenEvent, type UnSubscriber } from '../handlers/mod.ts';
import type { PhysicalMonitor } from '@seelen-ui/types';
import { List } from '../utils/List.ts';
import { newFromInvoke, newOnEvent } from '../utils/State.ts';

export class ConnectedMonitorList extends List<PhysicalMonitor> {
  static getAsync(): Promise<ConnectedMonitorList> {
    return newFromInvoke(this, SeelenCommand.SystemGetMonitors);
  }

  static onChange(cb: (payload: ConnectedMonitorList) => void): Promise<UnSubscriber> {
    return newOnEvent(cb, this, SeelenEvent.SystemMonitorsChanged);
  }
}
