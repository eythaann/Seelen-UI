import type { NotificationsToolbarItem } from "@seelen-ui/lib/types";
import { Popover } from "antd";
import { useSelector } from "react-redux";

import { Item } from "../../item/infra/infra.tsx";

import { Selectors } from "../../shared/store/app.ts";

import type { RootState } from "../../shared/store/domain.ts";

import { ArrivalPreview } from "./ArrivalPreview.tsx";

interface Props {
  module: NotificationsToolbarItem;
}

export function NotificationsModule({ module }: Props) {
  const count = useSelector((state: RootState) => Selectors.notifications(state).length);

  return (
    <Popover open arrow={false} content={<ArrivalPreview />}>
      <Item extraVars={{ count }} module={module} />
    </Popover>
  );
}
