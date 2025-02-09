import { SeelenCommand, SeelenWegSide } from '@seelen-ui/lib';
import { convertFileSrc, invoke } from '@tauri-apps/api/core';
import moment from 'moment';
import { memo, useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useSelector } from 'react-redux';

import { BackgroundByLayersV2 } from '../../../components/BackgroundByLayers/infra';
import { LAZY_CONSTANTS, updatePreviews } from '../../shared/utils/infra';

import { Selectors } from '../../shared/store/app';
import { parseCommand } from 'src/apps/shared/Command';
import { useIcon, useWindowFocusChange } from 'src/apps/shared/hooks';

import { PinnedWegItem, TemporalWegItem } from '../../shared/store/domain';

import { AnimatedPopover } from '../../../../shared/components/AnimatedWrappers';
import { cx } from '../../../../shared/styles';
import { WithContextMenu } from '../../../components/WithContextMenu';
import { DraggableItem } from './DraggableItem';
import { getUserApplicationContextMenu } from './UserApplicationContextMenu';
import { UserApplicationPreview } from './UserApplicationPreview';

interface Props {
  item: PinnedWegItem | TemporalWegItem;
  // This will be triggered in case preview or context menu is opened from this item, or both of them closed.
  onAssociatedViewOpenChanged?: (isOpen: boolean) => void;
}

export const UserApplication = memo(({ item, onAssociatedViewOpenChanged }: Props) => {
  const [openPreview, setOpenPreview] = useState(false);
  const [openContextMenu, setOpenContextMenu] = useState(false);
  const [blockUntil, setBlockUntil] = useState(moment(new Date()));

  const notifications = useSelector(Selectors.notifications);
  const devTools = useSelector(Selectors.devTools);
  const settings = useSelector(Selectors.settings);
  const focusedApp = useSelector(Selectors.focusedApp);

  const iconSrc =
    useIcon({
      path: item.path,
      umid: item.umid,
    }) || convertFileSrc(LAZY_CONSTANTS.MISSING_ICON_PATH);

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
    if (openPreview) {
      updatePreviews(item.windows.map((w) => w.handle));
    }
  }, [openPreview]);

  useEffect(() => {
    if (onAssociatedViewOpenChanged) {
      onAssociatedViewOpenChanged(openPreview || openContextMenu);
    }
  }, [openPreview || openContextMenu]);

  const notificationsCount = notifications.filter((n) => n.appUmid === item.umid).length;
  return (
    <DraggableItem
      item={item}
      className={cx({ 'associated-view-open': openPreview || openContextMenu })}
    >
      <WithContextMenu
        items={getUserApplicationContextMenu(t, item, devTools, iconSrc) || []}
        onOpenChange={(isOpen) => {
          setOpenContextMenu(isOpen);
          if (openPreview && isOpen) {
            setOpenPreview(false);
          }
        }}
      >
        <AnimatedPopover
          animationDescription={{
            maxAnimationTimeMs: 500,
            openAnimationName: 'weg-item-preview-container-open',
            closeAnimationName: 'weg-item-preview-container-close',
          }}
          open={openPreview}
          mouseEnterDelay={0.4}
          placement={calculatePlacement(settings.position)}
          onOpenChange={(open) =>
            setOpenPreview(open && !openContextMenu && moment(new Date()) > blockUntil)
          }
          trigger="hover"
          arrow={false}
          content={
            <BackgroundByLayersV2
              className={cx('weg-item-preview-container', settings.position.toLowerCase())}
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
                const { program, args } = parseCommand(item.relaunchCommand);
                invoke(SeelenCommand.Run, { program, args, workingDir: item.relaunchIn });
              } else {
                invoke(SeelenCommand.WegToggleWindowState, {
                  hwnd: window.handle,
                  wasFocused: focusedApp?.hwnd === window.handle,
                });
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
            <img className="weg-item-icon" src={iconSrc} />
            {notificationsCount > 0 && <div className="weg-item-badge">{notificationsCount}</div>}
            <div
              className={cx('weg-item-open-sign', {
                'weg-item-open-sign-active': !!item.windows.length,
                'weg-item-open-sign-focused': item.windows.some(
                  (w) => w.handle === focusedApp?.hwnd,
                ),
              })}
            />
          </div>
        </AnimatedPopover>
      </WithContextMenu>
    </DraggableItem>
  );
});
