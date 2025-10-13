import type { PowerToolbarItem } from "@seelen-ui/lib/types";
import { useSelector } from "react-redux";

import { Item } from "../item/infra/infra.tsx";

import { Selectors } from "../shared/store/app.ts";

interface Props {
  module: PowerToolbarItem;
}

export function PowerModule({ module }: Props) {
  const power = useSelector(Selectors.powerStatus);
  const powerMode = useSelector(Selectors.powerPlan);
  const batteries = useSelector(Selectors.batteries);

  return (
    <Item
      extraVars={{
        power,
        powerMode,
        batteries,
      }}
      module={module}
    />
  );
}
