import { emit } from '@tauri-apps/api/event';
import { Popover } from 'antd';
import React, { useEffect, useState } from 'react';
import { useSelector } from 'react-redux';
import { NotificationsTM, useWindowFocusChange } from 'seelen-core';

import { Item } from '../../item/infra/infra';

import { Selectors } from '../../shared/store/app';

import { RootState } from '../../shared/store/domain';

import { ArrivalPreview } from './ArrivalPreview';
import { Notifications } from './Notifications';

interface Props {
  module: NotificationsTM;
}

export function NotificationsModule({ module }: Props) {
  const [openPreview, setOpenPreview] = useState(false);
  const count = useSelector((state: RootState) => Selectors.notifications(state).length);

  useWindowFocusChange((focused) => {
    if (!focused) {
      setOpenPreview(false);
    }
  });

  useEffect(() => {
    emit('register-notifications-events');
  }, []);

  return (
    <React.Fragment>
      <Popover
        open={openPreview}
        trigger="click"
        onOpenChange={setOpenPreview}
        arrow={false}
        content={<Notifications />}
      >
        <Item extraVars={{ count }} module={module} />
      </Popover>
      <Popover
        open={!openPreview}
        arrow={false}
        content={<ArrivalPreview />}
      >

      </Popover>
    </React.Fragment>
  );
}
