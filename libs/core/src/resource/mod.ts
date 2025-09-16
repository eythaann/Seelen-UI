import type { ResourceKind, ResourceStatus } from "@seelen-ui/types";
import type { Enum } from "../utils/enums.ts";

const ResourceKind: Enum<ResourceKind> = {
  IconPack: "IconPack",
  Theme: "Theme",
  Widget: "Widget",
  Plugin: "Plugin",
  Wallpaper: "Wallpaper",
  SoundPack: "SoundPack",
};

const ResourceStatus: Enum<ResourceStatus> = {
  Draft: "Draft",
  Reviewing: "Reviewing",
  Rejected: "Rejected",
  Published: "Published",
  Deleted: "Deleted",
};

export { ResourceKind, ResourceStatus };
