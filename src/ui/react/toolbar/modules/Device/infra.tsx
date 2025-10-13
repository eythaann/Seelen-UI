import type { DeviceToolbarItem } from "@seelen-ui/lib/types";

import { Item } from "../item/infra/infra.tsx";

interface Props {
  module: DeviceToolbarItem;
}

export function DeviceModule({ module }: Props) {
  return <Item extraVars={{}} module={module} />;
}
