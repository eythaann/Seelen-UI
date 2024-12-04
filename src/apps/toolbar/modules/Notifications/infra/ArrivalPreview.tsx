import { invoke } from '@tauri-apps/api/core';
import { Button } from 'antd';
import { AnimatePresence, motion } from 'framer-motion';
import moment from 'moment';
import { useEffect, useState } from 'react';
import { useSelector } from 'react-redux';
import { SeelenCommand } from 'seelen-core';

import { BackgroundByLayersV2 } from '../../../../seelenweg/components/BackgroundByLayers/infra';

import { Selectors } from '../../shared/store/app';
import { useTimeout } from 'src/apps/shared/hooks';

import { AppNotification } from '../../shared/store/domain';

import { Icon } from '../../../../shared/components/Icon';

// Difference between Windows epoch (1601) and Unix epoch (1970) in milliseconds
const EPOCH_DIFF_MILLISECONDS = 11644473600000n;
const notificationArrivalViewTime = 10 * 1000;

function WindowsDateFileTimeToDate(fileTime: bigint) {
  return new Date(Number(fileTime / 10000n - EPOCH_DIFF_MILLISECONDS));
}

export function ArrivalPreview() {
  const notifications = useSelector(Selectors.notifications);
  const [currentNotificationPreviewSet, setCurrentNotificationPreviewSet] = useState<AppNotification[]>([]);

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
    <BackgroundByLayersV2 className="notification-arrival" onContextMenu={(e) => e.stopPropagation()}>
      <AnimatePresence>
        {currentNotificationPreviewSet &&
          currentNotificationPreviewSet.map((notification) => {
            return (
              <motion.div
                className="notification"
                key={notification.id}
                animate={{ x: '0%', opacity: 1 }}
                exit={{ x: '100%', opacity: 0 }}
                initial={{ x: '100%', opacity: 1 }}
                transition={{ duration: 0.4 }}
              >
                <div className="notification-header">
                  <div className="notification-header-info">
                    <Icon iconName="TbNotification" />
                    <div>{notification.app_name}</div>
                    <span>-</span>
                    <div>
                      {moment(WindowsDateFileTimeToDate(BigInt(notification.date))).fromNow()}
                    </div>
                  </div>
                  <Button
                    size="small"
                    type="text"
                    onClick={() => {
                      invoke(SeelenCommand.NotificationsClose, { id: notification.id }).catch(
                        console.error,
                      );
                    }}
                  >
                    <Icon iconName="IoClose" />
                  </Button>
                </div>
                <div className="notification-body">
                  <h2 className="notification-body-title">{notification.body[0]}</h2>
                  {notification.body.slice(1).map((body, idx) => (
                    <p key={idx}>{body}</p>
                  ))}
                </div>
              </motion.div>
            );
          })}
      </AnimatePresence>
    </BackgroundByLayersV2>
  );
}
