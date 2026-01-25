import type { UserToolbarItem } from "@seelen-ui/lib/types";
import { useSelector } from "react-redux";

import { Item } from "../item/infra/infra.tsx";

import { Selectors } from "../shared/store/app.ts";

interface Props {
  module: UserToolbarItem;
}

export function UserModule({ module }: Props) {
  const user = useSelector(Selectors.user);

  return (
    <Item
      module={module}
      extraVars={{ user }}
    />
  );
}
