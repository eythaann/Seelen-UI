// This file is for testing, not final implementation yet.

import { WidgetId } from "@seelen-ui/lib/types";
import { Icon } from "@shared/components/Icon";
import { Button, Modal } from "antd";
import { ReactNode, useState } from "react";

import { WidgetConfiguration } from "../../resources/Widget/View";

interface Props {
  widgetId: WidgetId;
  monitorId: string;
  title: ReactNode;
}

export function WidgetSettingsModal({ widgetId, monitorId, title }: Props) {
  const [open, setOpen] = useState(false);

  return (
    <>
      <Modal
        open={open}
        onCancel={() => setOpen(false)}
        title={title}
        footer={null}
        centered
      >
        <WidgetConfiguration widgetId={widgetId} monitorId={monitorId} />
      </Modal>
      <Button type="default" onClick={() => setOpen(true)}>
        <Icon iconName="RiSettings4Fill" />
      </Button>
    </>
  );
}
