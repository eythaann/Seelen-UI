import { invoke, SeelenCommand, SeelenEvent, type UnSubscriber } from "../../handlers/mod.ts";
import { List } from "../../utils/List.ts";
import type { BluetoothDevice } from "@seelen-ui/types";
import { newFromInvoke, newOnEvent } from "../../utils/State.ts";

export class BluetoothDevices extends List<BluetoothDevice> {
  static getAsync(): Promise<BluetoothDevices> {
    return newFromInvoke(this, SeelenCommand.GetBluetoothDevices);
  }

  static onChange(cb: (payload: BluetoothDevices) => void): Promise<UnSubscriber> {
    return newOnEvent(cb, this, SeelenEvent.BluetoothDevicesChanged);
  }

  static async discover(): Promise<void> {
    return await invoke(SeelenCommand.StartBluetoothScanning);
  }

  static async stopDiscovery(): Promise<void> {
    return await invoke(SeelenCommand.StopBluetoothScanning);
  }

  static default(): BluetoothDevices {
    return new this([]);
  }
}
