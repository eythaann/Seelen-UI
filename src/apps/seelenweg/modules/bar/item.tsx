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
import cs from './infra.module.css';

import { Selectors } from '../shared/store/app';

import { PinnedApp } from '../shared/store/domain';

interface Props {
  item: PinnedApp /* | SpecialApp */;
  initialSize: number;
}

export const WegItem = memo(({ item, initialSize }: Props) => {
  const theme = useSelector(Selectors.theme.seelenweg);
  const spaceBetweenItems = useSelector(Selectors.settings.spaceBetweenItems);

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

  return (
    <Reorder.Item
      as="div"
      value={item}
      drag="x"
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
          <>
            <BackgroundByLayers styles={theme.contextMenu.background} />
            <Menu style={theme.contextMenu.content} onMouseMoveCapture={(e) => e.stopPropagation()} items={getMenuForItem(item)} />
          </>
        )}
      >
        <Popover
          open={openPreview && !!item.opens.length}
          mouseEnterDelay={0.2}
          placement="top"
          onOpenChange={setOpenPreview}
          trigger="hover"
          arrow={false}
          content={
            <>
              <BackgroundByLayers styles={theme.preview.background} />
              <div
                className={cs.previewContainer}
                style={{
                  ...theme.preview.content,
                  gap: spaceBetweenItems + 'px',
                }}
                onMouseMoveCapture={(e) => e.stopPropagation()}
              >
                {item.opens.map((hwnd) => (
                  <WegPreview key={hwnd} hwnd={hwnd} />
                ))}
              </div>
            </>
          }
        >
          <animated.button
            className={cs.item}
            style={style}
            onClick={() => {
              if (!isDragging.current) {
                let hwnd = item.opens[0] || 0;
                invoke('weg_toggle_window_state', { hwnd, exePath: item.execution_path });
              }
            }}
          >
            <BackgroundByLayers styles={theme.items.background} />
            <img src={item.icon} style={theme.items.icon} draggable={false} />
          </animated.button>
        </Popover>
      </Dropdown>
    </Reorder.Item>
  );
});
