import { SeelenCommand, SeelenWegSide } from '@seelen-ui/lib';
import { invoke } from '@tauri-apps/api/core';
import { emit } from '@tauri-apps/api/event';
import moment from 'moment';
import { memo, useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useSelector } from 'react-redux';

import { BackgroundByLayersV2 } from '../../../components/BackgroundByLayers/infra';

import { Selectors } from '../../shared/store/app';
import { FileIcon } from 'src/apps/shared/components/Icon';
import { useWindowFocusChange } from 'src/apps/shared/hooks';

import { PinnedWegItem, TemporalWegItem } from '../../shared/store/domain';

import { AnimatedPopover } from '../../../../shared/components/AnimatedWrappers';
import { cx } from '../../../../shared/styles';
import { WithContextMenu } from '../../../components/WithContextMenu';
import { $settings } from '../../shared/state/mod';
import { DraggableItem } from './DraggableItem';
import { getUserApplicationContextMenu } from './UserApplicationContextMenu';
import { UserApplicationPreview } from './UserApplicationPreview';

interface Props {
  item: PinnedWegItem | TemporalWegItem;
}

export const UserApplication = memo(({ item }: Props) => {
  const [openPreview, setOpenPreview] = useState(false);
  const [openContextMenu, setOpenContextMenu] = useState(false);
  const [blockUntil, setBlockUntil] = useState(moment(new Date()));

  const notifications = useSelector(Selectors.notifications);
  const devTools = useSelector(Selectors.devTools);
  const focusedApp = useSelector(Selectors.focusedApp);

  const { t } = useTranslation();
  const calculatePlacement = (position: any) => {
    switch (position) {
      case SeelenWegSide.Bottom: {
        return 'top';
      }
      case SeelenWegSide.Top: {
        return 'bottom';
      }
      case SeelenWegSide.Left: {
        return 'right';
      }
      case SeelenWegSide.Right: {
        return 'left';
      }
      default: {
        throw new Error('Not Implemented!');
      }
    }
  };

  useWindowFocusChange((focused) => {
    if (!focused) {
      setBlockUntil(moment(new Date()).add(1, 'second'));
      setOpenPreview(false);
      setOpenContextMenu(false);
    }
  });

  useEffect(() => {
    if (openPreview && $settings.value.thumbnailGenerationEnabled) {
      invoke(SeelenCommand.WegRequestUpdatePreviews, {
        handles: item.windows.map((w) => w.handle),
      });
    }
  }, [openPreview]);

  const notificationsCount = notifications.filter((n) => n.appUmid === item.umid).length;
  const itemLabel = $settings.value.showWindowTitle && item.windows.length ? item.windows[0]!.title : null;

  return (
    <DraggableItem
      item={item}
      className={cx({ 'associated-view-open': openPreview || openContextMenu })}
    >
      <WithContextMenu
        items={getUserApplicationContextMenu(t, item, devTools, $settings.value.showEndTask) || []}
        onOpenChange={(isOpen) => {
          setOpenContextMenu(isOpen);
          if (openPreview && isOpen) {
            setOpenPreview(false);
          }
        }}
      >
        <AnimatedPopover
          animationDescription={{
            openAnimationName: 'weg-item-preview-container-open',
            closeAnimationName: 'weg-item-preview-container-close',
          }}
          open={openPreview}
          mouseEnterDelay={0.4}
          placement={calculatePlacement($settings.value.position)}
          onOpenChange={(open) =>
            setOpenPreview(open && !openContextMenu && moment(new Date()) > blockUntil)
          }
          trigger="hover"
          content={
            <BackgroundByLayersV2
              className={cx('weg-item-preview-container', $settings.value.position.toLowerCase())}
              onMouseMoveCapture={(e) => e.stopPropagation()}
              onContextMenu={(e) => {
                e.stopPropagation();
                e.preventDefault();
              }}
              prefix="preview"
            >
              <div className="weg-item-preview-scrollbar">
                {item.windows.map((window) => (
                  <UserApplicationPreview
                    key={window.handle}
                    title={window.title}
                    hwnd={window.handle}
                    isFocused={focusedApp?.hwnd === window.handle}
                  />
                ))}
                {item.windows.length === 0 && (
                  <div className="weg-item-display-name">{item.displayName}</div>
                )}
              </div>
            </BackgroundByLayersV2>
          }
        >
          <div
            className="weg-item"
            onClick={() => {
              let window = item.windows[0];
              if (!window) {
                invoke(SeelenCommand.Run, {
                  program: item.relaunchProgram,
                  args: item.relaunchArgs,
                  workingDir: item.relaunchIn,
                });
              } else {
                const wasFocused = focusedApp?.hwnd === window.handle;
                invoke(SeelenCommand.WegToggleWindowState, {
                  hwnd: window.handle,
                  wasFocused,
                });
                // this fix an issue of persisting focused colors when minimizing from dock
                if (wasFocused) {
                  emit('hidden::remove-focused-color');
                }
              }
            }}
            onAuxClick={(e) => {
              let window = item.windows[0];
              if (e.button === 1 && window) {
                invoke(SeelenCommand.WegCloseApp, { hwnd: window.handle });
              }
            }}
            onContextMenu={(e) => e.stopPropagation()}
          >
            <BackgroundByLayersV2 prefix="item" />
            <FileIcon className="weg-item-icon" path={item.path} umid={item.umid} />
            {itemLabel && (
              <div className="weg-item-title">{itemLabel}</div>
            )}
            {notificationsCount > 0 && (
              <div className="weg-item-notification-badge">{notificationsCount}</div>
            )}
            {$settings.value.showInstanceCounter && item.windows.length > 1 && (
              <div className="weg-item-instance-counter-badge">{item.windows.length}</div>
            )}
            {!$settings.value.showWindowTitle && (
              <div
                className={cx('weg-item-open-sign', {
                  'weg-item-open-sign-active': !!item.windows.length,
                  'weg-item-open-sign-focused': item.windows.some(
                    (w) => w.handle === focusedApp?.hwnd,
                  ),
                })}
              />
            )}
          </div>
        </AnimatedPopover>
      </WithContextMenu>
    </DraggableItem>
  );
});
