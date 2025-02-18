import { SeelenCommand } from '@seelen-ui/lib';
import { invoke } from '@tauri-apps/api/core';
import { Button } from 'antd';
import { motion } from 'framer-motion';
import moment from 'moment';

import { FileIcon, Icon } from 'src/apps/shared/components/Icon';

import { AppNotification } from '../domain';

interface Props {
  notification: AppNotification;
}

// Difference between Windows epoch (1601) and Unix epoch (1970) in milliseconds
const EPOCH_DIFF_MILLISECONDS = 11644473600000n;

function WindowsDateFileTimeToDate(fileTime: bigint) {
  return new Date(Number(fileTime / 10000n - EPOCH_DIFF_MILLISECONDS));
}

export function Notification({ notification }: Props) {
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
          <FileIcon className="notification-icon" umid={notification.appUmid} />
          <div>{notification.appName}</div>
          <span>-</span>
          <div>{moment(WindowsDateFileTimeToDate(BigInt(notification.date))).fromNow()}</div>
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
  );
}
