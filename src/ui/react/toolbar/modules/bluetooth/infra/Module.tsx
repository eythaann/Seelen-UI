import type { BluetoothDevice, BluetoothToolbarItem } from "@seelen-ui/lib/types";
import { useState } from "react";
import { useSelector } from "react-redux";

import { Item } from "../../item/infra/infra.tsx";

import { Selectors } from "../../shared/store/app.ts";

import { WithBluetoothSelector } from "./BluetoothSelector.tsx";

interface Props {
  active?: boolean;
  module: BluetoothToolbarItem;
}

function BluetoothModuleItem({ module, active, ...rest }: Props) {
  const bluetoothDevices: BluetoothDevice[] = useSelector(
    Selectors.bluetoothDevices,
  );
  const connectedDevices = bluetoothDevices.filter((item) => item.connected);

  return (
    <Item
      {...rest}
      extraVars={{
        devices: bluetoothDevices,
        connectedDevices,
      }}
      module={module}
      active={active}
    />
  );
}

export function BluetoothModule({ module }: Props) {
  const [open, setOpen] = useState(false);

  return module.withBluetoothSelector
    ? (
      <WithBluetoothSelector setActive={setOpen}>
        <BluetoothModuleItem module={module} active={open} />
      </WithBluetoothSelector>
    )
    : <BluetoothModuleItem module={module} />;
}
