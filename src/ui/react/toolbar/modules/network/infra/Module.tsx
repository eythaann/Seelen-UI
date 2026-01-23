import type { NetworkToolbarItem } from "@seelen-ui/lib/types";
import { useSelector } from "react-redux";

import { Item } from "../../item/infra/infra.tsx";

import { Selectors } from "../../shared/store/app.ts";

interface Props {
  module: NetworkToolbarItem;
}

export function NetworkModule({ module, ...rest }: Props) {
  const networkAdapters = useSelector(Selectors.networkAdapters);
  const defaultIp = useSelector(Selectors.networkLocalIp);
  const online = useSelector(Selectors.online);

  const usingAdapter = networkAdapters.find((i) => i.ipv4 === defaultIp) || null;

  return (
    <Item
      {...rest}
      extraVars={{
        online,
        interfaces: networkAdapters,
        usingInterface: usingAdapter,
      }}
      module={module}
    />
  );
}
