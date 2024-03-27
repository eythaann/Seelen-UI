import { ExtraCallbacksOnLeave } from '../../events';
import { animated, useSpring } from '@react-spring/web';
import { invoke } from '@tauri-apps/api/core';
import { Dropdown } from 'antd';
import { Reorder } from 'framer-motion';
import { memo, useEffect, useRef, useState } from 'react';
import { useSelector } from 'react-redux';

import { BackgroundByLayers } from '../../components/BackgrounByLayers/infra';
import { TooltipWrap } from '../../components/TooltipWrap/infra';
import cs from './infra.module.css';

import { Selectors } from '../shared/store/app';
import { getMenuForItem } from './app';

import { OpenApp, PinnedApp } from '../shared/store/domain';

interface Props {
  item: OpenApp | PinnedApp;
  initialSize: number;
}

export const WegItem = memo(({ item, initialSize }: Props) => {
  const { background } = useSelector(Selectors.theme.seelenweg.items);
  const [open, setOpen] = useState(false);
  const isDragging = useRef(false);

  const style = useSpring({
    from: { width: 0, height: 0 },
    to: { width: initialSize, height: initialSize },
    config: {
      clamp: true,
      mass: 5,
      tension: 170,
      friction: 1,
      velocity: 1,
    },
  });

  useEffect(() => {
    ExtraCallbacksOnLeave.add(() => {
      setOpen(false);
    });
  }, []);

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
        }, 150);
      }}
    >
      <Dropdown
        getPopupContainer={() => document.getElementById('root')!}
        placement="top"
        open={open}
        onOpenChange={setOpen}
        trigger={['contextMenu']}
        menu={{
          items: getMenuForItem(item),
        }}
      >
        <animated.button
          className={cs.item}
          style={style}
          onClick={() => {
            if (!isDragging.current) {
              invoke('weg_toggle_window_state', { hwnd: item.hwnd || 0, exePath: item.exe });
            }
          }}
        >
          <BackgroundByLayers styles={background} />
          <TooltipWrap showToltip={!open} text={item.title}>
            <img src={item.icon} draggable={false} />
          </TooltipWrap>
        </animated.button>
      </Dropdown>
    </Reorder.Item>
  );
});
