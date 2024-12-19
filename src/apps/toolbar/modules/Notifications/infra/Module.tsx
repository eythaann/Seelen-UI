import { NotificationsToolbarItem } from '@seelen-ui/lib/types';
import { emit } from '@tauri-apps/api/event';
import { Popover } from 'antd';
import { useEffect, useState } from 'react';
import { useSelector } from 'react-redux';

import { Item } from '../../item/infra/infra';

import { Selectors } from '../../shared/store/app';
import { useWindowFocusChange } from 'src/apps/shared/hooks';

import { RootState } from '../../shared/store/domain';

import { ArrivalPreview } from './ArrivalPreview';
import { Notifications } from './Notifications';

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

  useEffect(() => {
    emit('register-notifications-events');
  }, []);

  return (
    <Popover open={!openPreview} arrow={false} content={<ArrivalPreview />}>
      <Popover
        open={openPreview}
        trigger="click"
        onOpenChange={setOpenPreview}
        arrow={false}
        content={<Notifications />}
      >
        <Item extraVars={{ count }} module={module} />
      </Popover>
    </Popover>
  );
}
