import { AnimatePresence } from 'framer-motion';
import moment from 'moment';
import { useEffect, useState } from 'react';
import { useSelector } from 'react-redux';

import { BackgroundByLayersV2 } from '../../../../seelenweg/components/BackgroundByLayers/infra';

import { Selectors } from '../../shared/store/app';
import { useTimeout } from 'src/apps/shared/hooks';

import { AppNotification } from '../domain';

import { Notification } from './Notification';

// Difference between Windows epoch (1601) and Unix epoch (1970) in milliseconds
const EPOCH_DIFF_MILLISECONDS = 11644473600000n;
const notificationArrivalViewTime = 10 * 1000;

function WindowsDateFileTimeToDate(fileTime: bigint) {
  return new Date(Number(fileTime / 10000n - EPOCH_DIFF_MILLISECONDS));
}

export function ArrivalPreview() {
  const notifications = useSelector(Selectors.notifications);
  const [currentNotificationPreviewSet, setCurrentNotificationPreviewSet] = useState<
    AppNotification[]
  >([]);

  useTimeout(
    () => {
      setCurrentNotificationPreviewSet(
        notifications.filter((notification) => {
          const arrivalDate = WindowsDateFileTimeToDate(BigInt(notification.date));

          return moment(Date.now()).diff(arrivalDate, 'seconds') < 10;
        }),
      );
    },
    notificationArrivalViewTime,
    [currentNotificationPreviewSet],
  );

  useEffect(() => {
    setCurrentNotificationPreviewSet(
      notifications.filter((notification) => {
        const arrivalDate = WindowsDateFileTimeToDate(BigInt(notification.date));

        return moment(Date.now()).diff(arrivalDate, 'seconds') < 10;
      }),
    );
  }, [notifications]);

  return (
    <BackgroundByLayersV2
      className="notification-arrival"
      onContextMenu={(e) => e.stopPropagation()}
    >
      <AnimatePresence>
        {currentNotificationPreviewSet.map((notification) => (
          <Notification key={notification.id} notification={notification} />
        ))}
      </AnimatePresence>
    </BackgroundByLayersV2>
  );
}
