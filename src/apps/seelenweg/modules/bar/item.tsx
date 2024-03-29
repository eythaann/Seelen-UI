import { ExtraCallbacksOnLeave } from '../../events';
import { WegPreview } from './preview';
import { animated, useSpring } from '@react-spring/web';
import { invoke } from '@tauri-apps/api/core';
import { Dropdown, Popover } from 'antd';
import { Reorder } from 'framer-motion';
import { memo, useEffect, useRef, useState } from 'react';
import { useSelector } from 'react-redux';

import { BackgroundByLayers } from '../../components/BackgrounByLayers/infra';
import cs from './infra.module.css';

import { Selectors } from '../shared/store/app';
import { getMenuForItem } from './app';

import { PinnedApp } from '../shared/store/domain';

interface Props {
  item: PinnedApp /* | SpecialApp */;
  initialSize: number;
}

export const WegItem = memo(({ item, initialSize }: Props) => {
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
        }, 150);
      }}
    >
      <Dropdown
        placement="topLeft"
        open={openContextMenu}
        onOpenChange={setOpenContextMenu}
        trigger={['contextMenu']}
        menu={{
          onMouseMoveCapture: (e) => e.stopPropagation(),
          items: getMenuForItem(item),
        }}
      >
        <Popover
          open={openPreview}
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
                style={theme.preview.content}
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
                invoke('weg_toggle_window_state', { hwnd, exePath: item.exe });
              }
            }}
          >
            <BackgroundByLayers styles={theme.items.background} />
            <img src={item.icon} draggable={false} />
          </animated.button>
        </Popover>
      </Dropdown>
    </Reorder.Item>
  );
});
