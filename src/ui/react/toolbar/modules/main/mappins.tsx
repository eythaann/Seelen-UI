import { type ToolbarItem, type ToolbarItem2, ToolbarModuleType } from "@seelen-ui/lib/types";
import type { AnyComponent } from "preact";
import { memo } from "react";

import { BluetoothModule } from "../bluetooth/infra/Module.tsx";
import { DateModule } from "../Date/infra.tsx";
import { DeviceModule } from "../Device/infra.tsx";
import { AppsItem, TextItem } from "../item/infra/infra.tsx";
import { KeyboardModule } from "../Keyboard/infra.tsx";
import { MediaModule } from "../media/infra/Module.tsx";
import { NetworkModule } from "../network/infra/Module.tsx";
import { NotificationsModule } from "../Notifications/infra/Module.tsx";
import { PowerModule } from "../Power/infra.tsx";
import { UserModule } from "../user/index.tsx";

import { $plugins } from "../shared/state/items.ts";
import { WorkspacesModule } from "../Workspaces/index.tsx";

const modulesByType: Record<
  ToolbarModuleType,
  AnyComponent<{ module: any }>
> = {
  [ToolbarModuleType.Text]: memo(TextItem),
  [ToolbarModuleType.Generic]: memo(AppsItem),
  [ToolbarModuleType.User]: memo(UserModule),
  [ToolbarModuleType.Date]: memo(DateModule),
  [ToolbarModuleType.Power]: memo(PowerModule),
  [ToolbarModuleType.Keyboard]: memo(KeyboardModule),
  [ToolbarModuleType.Settings]: memo(TextItem),
  [ToolbarModuleType.Workspaces]: memo(WorkspacesModule),
  [ToolbarModuleType.Bluetooth]: memo(BluetoothModule),
  [ToolbarModuleType.Network]: memo(NetworkModule),
  [ToolbarModuleType.Media]: memo(MediaModule),
  [ToolbarModuleType.Device]: memo(DeviceModule),
  [ToolbarModuleType.Notifications]: memo(NotificationsModule),
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

  return <Component key={module.id} module={module} />;
}
