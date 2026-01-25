import type { GenericToolbarItem, TextToolbarItem } from "@seelen-ui/lib/types";

import { InnerItem, type InnerItemProps } from "./Inner.tsx";
import { $focused } from "../../shared/state/windows.ts";

export function Item(props: InnerItemProps) {
  return <InnerItem {...props} />;
}

export function AppsItem({ module, ...rest }: { module: GenericToolbarItem }) {
  const window = $focused.value;
  return <Item {...rest} module={module} extraVars={{ window }} />;
}

export function TextItem({ module, ...rest }: { module: TextToolbarItem }) {
  return <Item {...rest} module={module} />;
}
