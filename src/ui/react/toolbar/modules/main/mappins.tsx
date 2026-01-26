import type { ToolbarItem, ToolbarItem2 } from "@seelen-ui/lib/types";

import { $plugins } from "../shared/state/items.ts";
import { Item } from "../item/infra/infra.tsx";

// item can be a toolbar plugin id or a toolbar module
export function componentByModule(entry: ToolbarItem2) {
  let module: ToolbarItem | undefined;

  if (typeof entry === "string") {
    module = $plugins.value.find((p) => p.id === entry)?.plugin as ToolbarItem | undefined;
    if (!module) {
      return null;
    }

    module = { ...module };
    module.id = entry;
  } else {
    module = entry;
  }

  return <Item key={module.id} module={module} />;
}
