import { cx } from '../../../../shared/styles';
import { WithContextMenu } from '../../../components/WithContextMenu';
import { getMenuForItem } from '../../bar/menu';
import { DraggableItem } from './DraggableItem';
import { UserApplicationPreview } from './UserApplicationPreview';
import { invoke } from '@tauri-apps/api/core';
import { Popover } from 'antd';
import { motion } from 'framer-motion';
import { memo, useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useSelector } from 'react-redux';

import { BackgroundByLayers } from '../../../components/BackgrounByLayers/infra';
import { useAppBlur } from '../../shared/hooks/infra';
import { updatePreviews } from '../../shared/utils/infra';

import { Selectors } from '../../shared/store/app';

import { RootState, SwPinnedApp, SwTemporalApp } from '../../shared/store/domain';

interface Props {
  item: SwPinnedApp | SwTemporalApp;
}

export const UserApplication = memo(({ item }: Props) => {
  const themeLayers = useSelector(Selectors.themeLayers);
  const size = useSelector(Selectors.settings.size);
  const isFocused = useSelector((state: RootState) => item.opens.includes(state.focusedHandle));

  const [openPreview, setOpenPreview] = useState(false);

  const { t } = useTranslation();

  useAppBlur(() => {
    setOpenPreview(false);
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
          placement="top"
          onOpenChange={(open) => setOpenPreview(open && !!item.opens.length)}
          trigger="hover"
          arrow={false}
          content={
            <div
              className="weg-item-preview-container"
              onMouseMoveCapture={(e) => e.stopPropagation()}
            >
              <BackgroundByLayers prefix="preview" layers={themeLayers.weg.preview.bg} />
              {item.opens.map((hwnd) => (
                <UserApplicationPreview key={hwnd} hwnd={hwnd} />
              ))}
            </div>
          }
        >
          <motion.div
            className="weg-item"
            initial={{ scale: 0 }}
            animate={{ scale: 1 }}
            style={{ height: size, aspectRatio: '1/1' }}
            onClick={() => {
              let hwnd = item.opens[0] || 0;
              invoke('weg_toggle_window_state', { hwnd, exePath: item.execution_path });
            }}
            onContextMenu={(e) => e.stopPropagation()}
          >
            <BackgroundByLayers prefix="item" layers={themeLayers.weg.items.bg} />
            <img className="weg-item-icon" src={item.icon} draggable={false} />
            <div
              className={cx('weg-item-open-sign', {
                'weg-item-open-sign-active': !!item.opens.length,
                'weg-item-open-sign-focused': isFocused,
              })}
            />
          </motion.div>
        </Popover>
      </WithContextMenu>
    </DraggableItem>
  );
});
