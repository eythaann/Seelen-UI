import { invoke } from '@tauri-apps/api/core';
import { Popover } from 'antd';
import { memo, useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useSelector } from 'react-redux';
import { SeelenCommand, SeelenWegSide, useWindowFocusChange } from 'seelen-core';

import { BackgroundByLayersV2 } from '../../../components/BackgroundByLayers/infra';
import { updatePreviews } from '../../shared/utils/infra';

import { Selectors } from '../../shared/store/app';

import {
  ExtendedPinnedWegItem,
  ExtendedTemporalWegItem,
  RootState,
} from '../../shared/store/domain';

import { cx } from '../../../../shared/styles';
import { WithContextMenu } from '../../../components/WithContextMenu';
import { getMenuForItem } from '../../bar/menu';
import { DraggableItem } from './DraggableItem';
import { UserApplicationPreview } from './UserApplicationPreview';

interface Props {
  item: ExtendedPinnedWegItem | ExtendedTemporalWegItem;
}

export const UserApplication = memo(({ item }: Props) => {
  const isFocused = useSelector(
    (state: RootState) => state.focusedApp && item.opens.includes(state.focusedApp.hwnd),
  );

  const [openPreview, setOpenPreview] = useState(false);
  const settings = useSelector(Selectors.settings);

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
      setOpenPreview(false);
    }
  });

  useEffect(() => {
    if (openPreview) {
      updatePreviews(item.opens);
    }
  }, [openPreview]);

  useEffect(() => {
    if (!item.opens.length) {
      setOpenPreview(false);
    }
  }, [item]);

  return (
    <DraggableItem item={item}>
      <WithContextMenu items={getMenuForItem(t, item) || []}>
        <Popover
          open={openPreview}
          mouseEnterDelay={0.4}
          placement={calculatePlacement(settings.position)}
          onOpenChange={(open) => setOpenPreview(open && !!item.opens.length)}
          trigger="hover"
          arrow={false}
          content={
            <BackgroundByLayersV2
              className="weg-item-preview-container"
              onMouseMoveCapture={(e) => e.stopPropagation()}
              prefix="preview"
            >
              <div className="weg-item-preview-scrollbar">
                {item.opens.map((hwnd) => (
                  <UserApplicationPreview key={hwnd} hwnd={hwnd} />
                ))}
              </div>
            </BackgroundByLayersV2>
          }
        >
          <div
            className="weg-item"
            onClick={() => {
              let hwnd = item.opens[0];
              if (!hwnd) {
                if (item.path.endsWith('.lnk')) {
                  invoke(SeelenCommand.OpenFile, { path: item.path });
                } else {
                  invoke(SeelenCommand.OpenFile, { path: item.execution_command });
                }
              } else {
                invoke(SeelenCommand.WegToggleWindowState, { hwnd });
              }
            }}
            onAuxClick={(e) => {
              let hwnd = item.opens[0];
              if (e.button === 1 && hwnd) {
                invoke(SeelenCommand.WegCloseApp, { hwnd });
              }
            }}
            onContextMenu={(e) => e.stopPropagation()}
          >
            <BackgroundByLayersV2 prefix="item" />
            <img className="weg-item-icon" src={item.icon} draggable={false} />
            <div
              className={cx('weg-item-open-sign', {
                'weg-item-open-sign-active': !!item.opens.length,
                'weg-item-open-sign-focused': isFocused,
              })}
            />
          </div>
        </Popover>
      </WithContextMenu>
    </DraggableItem>
  );
});
