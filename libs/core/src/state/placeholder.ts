import type { ToolbarItem, WorkspaceToolbarItemMode } from "@seelen-ui/types";
import type { Enum } from "../utils/enums.ts";

// =================================================================================
//    From here some enums as helpers like @seelen-ui/types only contains types
// =================================================================================

const ToolbarModuleType: Enum<ToolbarItem["type"]> = {
  Text: "text",
  Generic: "generic",
  Date: "date",
  Power: "power",
  Keyboard: "keyboard",
  Network: "network",
  Bluetooth: "bluetooth",
  Media: "media",
  User: "user",
  Notifications: "notifications",
  Device: "device",
  Settings: "settings",
  Workspaces: "workspaces",
};

const WorkspaceToolbarItemMode: Enum<WorkspaceToolbarItemMode> = {
  Dotted: "dotted",
  Named: "named",
  Numbered: "numbered",
};

export { ToolbarModuleType, WorkspaceToolbarItemMode };
