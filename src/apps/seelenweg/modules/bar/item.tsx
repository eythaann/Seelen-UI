import { cx } from '../../../utils/styles';
import { ExtraCallbacksOnLeave } from '../../events';
import { savePinnedItems } from '../shared/store/storeApi';
import { getMenuForItem } from './menu';
import { WegPreview } from './preview';
import { animated, useSpring } from '@react-spring/web';
import { invoke } from '@tauri-apps/api/core';
import { Dropdown, Menu, Popover } from 'antd';
import { Reorder } from 'framer-motion';
import { memo, useEffect, useRef, useState } from 'react';
import { useSelector } from 'react-redux';

import { BackgroundByLayers } from '../../components/BackgrounByLayers/infra';
import { store } from '../shared/store/infra';
import { updatePreviews } from '../shared/utils/infra';

import { Selectors } from '../shared/store/app';

import { App } from '../shared/store/domain';

interface Props {
  item: App;
  initialSize: number;
  isFocused: boolean;
}

export const WegItem = memo(({ item, initialSize, isFocused }: Props) => {
  const theme = useSelector(Selectors.theme.seelenweg);

  const [openContextMenu, setOpenContextMenu] = useState(false);
  const [openPreview, setOpenPreview] = useState(false);

  const isDragging = useRef(false);

  const [style, _api] = useSpring(
    {
      from: { width: 0, height: 0 },
      to: { width: initialSize, height: initialSize },
      config: {
        clamp: true,
        mass: 5,
        tension: 170,
        friction: 1,
        velocity: 1,
      },
    },
    [],
  );

  useEffect(() => {
    ExtraCallbacksOnLeave.add(() => {
      setOpenContextMenu(false);
    });
  }, []);

  useEffect(() => {
    if (openContextMenu) {
      setOpenPreview(false);
    }
  }, [openContextMenu]);

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
    <Reorder.Item
      as="div"
      value={item}
      onDragStart={() => {
        isDragging.current = true;
      }}
      onDragEnd={() => {
        setTimeout(() => {
          isDragging.current = false;
          savePinnedItems(store.getState());
        }, 150);
      }}
    >
      <Dropdown
        placement="topLeft"
        open={openContextMenu}
        onOpenChange={setOpenContextMenu}
        trigger={['contextMenu']}
        dropdownRender={() => (
          <div className="weg-context-menu-container">
            <BackgroundByLayers prefix="menu" styles={theme.contextMenu.backgroundLayers} />
            <Menu className="weg-context-menu" onMouseMoveCapture={(e) => e.stopPropagation()} items={getMenuForItem(item)} />
          </div>
        )}
      >
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
              <BackgroundByLayers prefix="preview" styles={theme.preview.backgroundLayers} />
              {item.opens.map((hwnd) => (
                <WegPreview key={hwnd} hwnd={hwnd} />
              ))}
            </div>
          }
        >
          <animated.button
            className="weg-item"
            style={style}
            onClick={() => {
              if (!isDragging.current) {
                let hwnd = item.opens[0] || 0;
                invoke('weg_toggle_window_state', { hwnd, exePath: item.execution_path });
              }
            }}
          >
            <BackgroundByLayers prefix="item" styles={theme.items.backgroundLayers} />
            <img className="weg-item-icon" src={item.icon} draggable={false} />
            <div className={cx('weg-item-open-sign', {
              'weg-item-open-sign-active': !!item.opens.length,
              'weg-item-open-sign-focused': isFocused,
            })} />
          </animated.button>
        </Popover>
      </Dropdown>
    </Reorder.Item>
  );
});
