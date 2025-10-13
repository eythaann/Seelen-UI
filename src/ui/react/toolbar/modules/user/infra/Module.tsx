import type { UserToolbarItem } from "@seelen-ui/lib/types";
import { useState } from "react";
import { useSelector } from "react-redux";

import { Item } from "../../item/infra/infra.tsx";

import { Selectors } from "../../shared/store/app.ts";

import { WithUserHome } from "./UserHome.tsx";

interface Props {
  module: UserToolbarItem;
  active?: boolean;
}

function UserModuleItem({ module, active, ...rest }: Props) {
  const user = useSelector(Selectors.user);
  return (
    <Item
      {...rest}
      active={active}
      module={module}
      extraVars={{ user }}
    />
  );
}

export function UserModule({ module }: Props) {
  const [isActive, setIsActive] = useState(false);

  return module.withUserFolder
    ? (
      <WithUserHome setOpen={setIsActive}>
        <UserModuleItem module={module} active={isActive} />
      </WithUserHome>
    )
    : <UserModuleItem module={module} />;
}
