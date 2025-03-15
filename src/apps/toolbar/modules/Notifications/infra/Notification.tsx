import { SeelenCommand } from '@seelen-ui/lib';
import { ToastBindingEntry, ToastImage } from '@seelen-ui/lib/types';
import { convertFileSrc, invoke } from '@tauri-apps/api/core';
import { Tooltip } from 'antd';
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

function parseUriToSrc(uri: string) {
  if (uri.startsWith('http://') || uri.startsWith('https://')) {
    return uri;
  }

  if (uri.startsWith('file:///')) {
    const encodedPath = uri.slice('file:///'.length);
    const decoded = decodeURIComponent(encodedPath);
    return convertFileSrc(decoded);
  }

  return convertFileSrc(uri);
}

export function Notification({ notification }: Props) {
  const toast = notification.content;
  const template = toast.visual.binding['@template'];
  const actions = toast.actions?.$value || [];

  let logoImage: ToastImage | null = null;
  let heroImage: ToastImage | null = null;
  const content: ToastBindingEntry[] = [];

  for (const entry of toast.visual.binding.$value) {
    if ('image' in entry) {
      if (
        entry.image['@placement'] === 'AppLogoOverride' ||
        (!entry.image['@placement'] && !logoImage && template.startsWith('ToastImageAndText'))
      ) {
        logoImage = entry.image;
        continue;
      }

      if (entry.image['@placement'] === 'Hero') {
        heroImage = entry.image;
        continue;
      }
    }
    content.push(entry);
  }

  return (
    <motion.div
      animate={{ x: '0%', opacity: 1 }}
      exit={{ x: '100%', opacity: 0 }}
      initial={{ x: '100%', opacity: 1 }}
      transition={{ duration: 0.4 }}
    >
      <div
        className="notification"
        onClick={() => {
          if (toast['@launch']) {
            invoke(SeelenCommand.OpenFile, { path: toast['@launch'] });
          }
        }}
      >
        <div className="notification-header">
          <div className="notification-header-info">
            <FileIcon className="notification-icon" umid={notification.appUmid} />
            <div>{notification.appName}</div>
            <span>-</span>
            <div>{moment(WindowsDateFileTimeToDate(BigInt(notification.date))).fromNow()}</div>
          </div>
          <button
            className="notification-header-close"
            onClick={(e) => {
              e.stopPropagation();
              invoke(SeelenCommand.NotificationsClose, { id: notification.id }).catch(
                console.error,
              );
            }}
          >
            <Icon iconName="IoClose" />
          </button>
        </div>
        <div className="notification-body">
          {logoImage && (
            <img
              src={parseUriToSrc(logoImage['@src'])}
              alt={logoImage['@alt'] || ''}
              className="notification-body-logo-image"
            />
          )}

          <div className="notification-body-content">
            {content.map((entry, index) => {
              if ('text' in entry) {
                return <p key={index}>{entry.text.$value}</p>;
              }

              if ('image' in entry && !entry.image['@placement']) {
                return (
                  <img
                    key={index}
                    src={parseUriToSrc(entry.image['@src'])}
                    alt={entry.image['@alt'] ?? undefined}
                  />
                );
              }

              return null;
            })}
          </div>

          {heroImage && (
            <img
              src={parseUriToSrc(heroImage['@src'])}
              alt={heroImage['@alt'] || ''}
              className="notification-body-hero-image"
            />
          )}
        </div>
        {!!actions.length && (
          <div className="notification-actions">
            {actions.map((entry, index) => {
              if ('action' in entry) {
                return (
                  <Tooltip key={index} title={entry.action['@hint-toolTip']}>
                    <button
                      className="notification-action"
                      onClick={(e) => {
                        e.stopPropagation();
                        if (entry.action['@activationType'] === 'Protocol') {
                          invoke(SeelenCommand.OpenFile, { path: entry.action['@arguments'] });
                        }
                      }}
                    >
                      {entry.action['@content']}
                    </button>
                  </Tooltip>
                );
              }
              return null;
            })}
          </div>
        )}
      </div>
    </motion.div>
  );
}
