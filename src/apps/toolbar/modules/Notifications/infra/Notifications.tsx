import { SeelenCommand } from '@seelen-ui/lib';
import { convertFileSrc, invoke } from '@tauri-apps/api/core';
import { Button } from 'antd';
import { AnimatePresence, motion } from 'framer-motion';
import moment from 'moment';
import { useSelector } from 'react-redux';

import { BackgroundByLayersV2 } from '../../../../seelenweg/components/BackgroundByLayers/infra';
import { LAZY_CONSTANTS } from '../../shared/utils/infra';

import { Selectors } from '../../shared/store/app';

import { AppNotification } from '../../shared/store/domain';

import { Icon } from '../../../../shared/components/Icon';

// Difference between Windows epoch (1601) and Unix epoch (1970) in milliseconds
const EPOCH_DIFF_MILLISECONDS = 11644473600000n;

function WindowsDateFileTimeToDate(fileTime: bigint) {
  return new Date(Number(fileTime / 10000n - EPOCH_DIFF_MILLISECONDS));
}

export function Notifications() {
  const notifications = useSelector(Selectors.notifications);

  return (
    <BackgroundByLayersV2 className="notifications" onContextMenu={(e) => e.stopPropagation()}>
      <div className="notifications-header">
        <span>Notifications</span>
        <Button
          size="small"
          onClick={() => {
            invoke(SeelenCommand.NotificationsCloseAll).catch(console.error);
          }}
        >
          Clear all
        </Button>
      </div>

      <div className="notifications-body">
        <AnimatePresence>
          {notifications.map((notification: AppNotification) => (
            <motion.div
              className="notification"
              key={notification.id}
              animate={{ x: '0%', opacity: 1 }}
              exit={{ x: '100%', opacity: 0 }}
              transition={{ duration: 0.4 }}
            >
              <div className="notification-header">
                <div className="notification-header-info">
                  <img className="notification-icon" src={convertFileSrc(notification.app_logo ? notification.app_logo : LAZY_CONSTANTS.MISSING_ICON_PATH)} />
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
                    invoke(SeelenCommand.NotificationsClose, { id: notification.id }).catch(console.error);
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
          ))}
        </AnimatePresence>

        {!notifications.length && (
          <motion.div
            className="notifications-empty"
            initial={{ opacity: 0, height: 0 }}
            animate={{ opacity: 1, height: 200 }}
            transition={{ duration: 0.2, delay: 0.4 }}
          >
            <p>No notifications</p>
          </motion.div>
        )}
      </div>
      <div className="notifications-footer">
        <Button
          size="small"
          type="text"
          onClick={() => {
            invoke(SeelenCommand.OpenFile, { path: 'ms-settings:notifications' }).catch(console.error);
          }}
        >
          Go to notifications settings
        </Button>
      </div>
    </BackgroundByLayersV2>
  );
}
