import type { BluetoothDevice, BluetoothToolbarItem } from "@seelen-ui/lib/types";
import { useSelector } from "react-redux";

import { Item } from "../../item/infra/infra.tsx";

import { Selectors } from "../../shared/store/app.ts";

interface Props {
  active?: boolean;
  module: BluetoothToolbarItem;
}

export function BluetoothModule({ module }: Props) {
  const bluetoothDevices: BluetoothDevice[] = useSelector(
    Selectors.bluetoothDevices,
  );
  const connectedDevices = bluetoothDevices.filter((item) => item.connected);

  return (
    <Item
      extraVars={{
        devices: bluetoothDevices,
        connectedDevices,
      }}
      module={module}
    />
  );
}
