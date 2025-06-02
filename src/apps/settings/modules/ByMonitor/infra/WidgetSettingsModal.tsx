// This file is for testing, not final implementation yet.

import { Button, Modal } from 'antd';
import { ReactNode, useState } from 'react';

import { Icon } from 'src/apps/shared/components/Icon';

import { WidgetConfiguration } from '../../resources/Widget/View';

interface Props {
  widgetId: string;
  monitorId: string;
  title: ReactNode;
}

export function WidgetSettingsModal({ widgetId, monitorId, title }: Props) {
  const [open, setOpen] = useState(false);

  return (
    <>
      <Modal open={open} onCancel={() => setOpen(false)} title={title} footer={null} centered>
        <WidgetConfiguration widgetId={widgetId} monitorId={monitorId} />
      </Modal>
      <Button type="default" onClick={() => setOpen(true)}>
        <Icon iconName="RiSettings4Fill" />
      </Button>
    </>
  );
}
