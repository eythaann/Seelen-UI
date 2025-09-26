import { ToolbarModuleType as ToolbarItemType } from "@seelen-ui/lib";
import { ToolbarItem, ToolbarItem2 } from "@seelen-ui/lib/types";
import { AnyComponent } from "preact";
import { memo } from "react";

import { BluetoothModule } from "../bluetooth/infra/Module";
import { DateModule } from "../Date/infra";
import { DeviceModule } from "../Device/infra";
import { GenericItem, TextItem } from "../item/infra/infra";
import { KeyboardModule } from "../Keyboard/infra";
import { MediaModule } from "../media/infra/Module";
import { NetworkModule } from "../network/infra/Module";
import { NotificationsModule } from "../Notifications/infra/Module";
import { PowerModule } from "../Power/infra";
import { SettingsModule } from "../Settings/infra";
import { UserModule } from "../user/infra/Module";

import { $plugins } from "../shared/state/items";
import { WorkspacesModule } from "../Workspaces";

const modulesByType: Record<
  ToolbarItem["type"],
  AnyComponent<{ module: any; value: any }>
> = {
  [ToolbarItemType.Text]: memo(TextItem),
  [ToolbarItemType.Generic]: memo(GenericItem),
  [ToolbarItemType.User]: memo(UserModule),
  [ToolbarItemType.Date]: memo(DateModule),
  [ToolbarItemType.Power]: memo(PowerModule),
  [ToolbarItemType.Keyboard]: memo(KeyboardModule),
  [ToolbarItemType.Settings]: memo(SettingsModule),
  [ToolbarItemType.Workspaces]: memo(WorkspacesModule),
  [ToolbarItemType.Bluetooth]: memo(BluetoothModule),
  [ToolbarItemType.Network]: memo(NetworkModule),
  [ToolbarItemType.Media]: memo(MediaModule),
  [ToolbarItemType.Device]: memo(DeviceModule),
  [ToolbarItemType.Notifications]: memo(NotificationsModule),
};

// item can be a toolbar plugin id or a toolbar module
export function componentByModule(entry: ToolbarItem2) {
  let module: ToolbarItem | undefined;

  if (typeof entry === "string") {
    module = $plugins.value.find((p) => p.id === entry)?.plugin as
      | ToolbarItem
      | undefined;
    if (!module) {
      return null;
    }
    module = { ...module };
    module.id = entry;
  } else {
    module = entry;
  }

  let Component = modulesByType[module.type];
  if (!Component) {
    return null;
  }
  return <Component key={module.id} module={module} value={entry} />;
}
