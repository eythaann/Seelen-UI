import type { NotificationsToolbarItem } from "@seelen-ui/lib/types";
import { AnimatedPopover } from "@shared/components/AnimatedWrappers";
import { useWindowFocusChange } from "@shared/hooks";
import { Popover } from "antd";
import { useState } from "react";
import { useSelector } from "react-redux";

import { Item } from "../../item/infra/infra.tsx";

import { Selectors } from "../../shared/store/app.ts";

import type { RootState } from "../../shared/store/domain.ts";

import { ArrivalPreview } from "./ArrivalPreview.tsx";
import { Notifications } from "./Notifications.tsx";

interface Props {
  module: NotificationsToolbarItem;
}

export function NotificationsModule({ module }: Props) {
  const [openPreview, setOpenPreview] = useState(false);
  const count = useSelector((state: RootState) => Selectors.notifications(state).length);

  useWindowFocusChange((focused) => {
    if (!focused) {
      setOpenPreview(false);
    }
  });

  return (
    <Popover open={!openPreview} arrow={false} content={<ArrivalPreview />}>
      <AnimatedPopover
        animationDescription={{
          openAnimationName: "notification-open",
          closeAnimationName: "notification-close",
        }}
        open={openPreview}
        trigger="click"
        onOpenChange={setOpenPreview}
        content={<Notifications />}
      >
        <Item extraVars={{ count }} module={module} active={openPreview} />
      </AnimatedPopover>
    </Popover>
  );
}
