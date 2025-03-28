import { AnimatePresence } from 'framer-motion';
import { debounce } from 'lodash';
import moment from 'moment';
import { useCallback, useEffect, useState } from 'react';
import { useSelector } from 'react-redux';

import { Selectors } from '../../shared/store/app';

import { AppNotification } from '../domain';

import { Notification } from './Notification';

// Difference between Windows epoch (1601) and Unix epoch (1970) in milliseconds
const EPOCH_DIFF_MILLISECONDS = 11644473600000n;
const notificationArrivalViewTime = 5 * 1000;

function WindowsDateFileTimeToDate(fileTime: bigint) {
  return new Date(Number(fileTime / 10000n - EPOCH_DIFF_MILLISECONDS));
}

export function ArrivalPreview() {
  const notifications = useSelector(Selectors.notifications);
  const [arrivals, setArrivals] = useState<AppNotification[]>([]);

  const updateArrivals = useCallback(() => {
    setArrivals(
      notifications.filter((notification) => {
        const arrivalDate = WindowsDateFileTimeToDate(BigInt(notification.date));
        return moment(Date.now()).diff(arrivalDate, 'seconds') < 5;
      }),
    );
  }, [notifications, setArrivals]);

  const cleanArrivals = useCallback(
    debounce(() => setArrivals([]), notificationArrivalViewTime),
    [setArrivals],
  );

  useEffect(() => {
    updateArrivals();
    cleanArrivals();
  }, [notifications]);

  return (
    <div className="notification-arrival" onContextMenu={(e) => e.stopPropagation()}>
      <AnimatePresence>
        {arrivals.map((notification) => (
          <Notification key={notification.id} notification={notification} />
        ))}
      </AnimatePresence>
    </div>
  );
}
