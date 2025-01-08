import { SeelenCommand } from '@seelen-ui/lib';
import { invoke } from '@tauri-apps/api/core';
import { Button } from 'antd';
import { AnimatePresence, motion } from 'framer-motion';
import { useSelector } from 'react-redux';

import { BackgroundByLayersV2 } from '../../../../seelenweg/components/BackgroundByLayers/infra';

import { Selectors } from '../../shared/store/app';

import { AppNotification } from '../domain';

import { Notification } from './Notification';

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
            <Notification key={notification.id} notification={notification} />
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
            invoke(SeelenCommand.OpenFile, { path: 'ms-settings:notifications' }).catch(
              console.error,
            );
          }}
        >
          Go to notifications settings
        </Button>
      </div>
    </BackgroundByLayersV2>
  );
}
