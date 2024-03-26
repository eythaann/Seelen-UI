import { updateHitbox } from '../../events';
import { convertFileSrc, invoke } from '@tauri-apps/api/core';
import { Tooltip } from 'antd';
import { CSSProperties, memo, MouseEvent, useEffect, useRef, useState } from 'react';
import { useSelector } from 'react-redux';

import cs from './infra.module.css';

import { cx } from '../../../settings/modules/shared/app/utils';
import { Selectors } from '../shared/store/app';

import { RootState } from '../shared/store/domain';

interface Item {
  style: CSSProperties;
  label: string;
  iconPath: string;
  exePath: string;
  hwnd: number;
}

const MAX_CURSOR_DISTANCE = 500;

function StoreAppsToItems(storeItems: RootState['apps'], size: number) {
  return storeItems.map<Item>((app) => ({
    label: app.state === 'Open' ? app.title : app.exe,
    iconPath: app.icon,
    hwnd: app.hwnd,
    exePath: app.exe,
    style: {
      width: size,
      height: size,
    },
  }));
}

interface SeelenWegBackgroundProps {
  styles: CSSProperties[];
}
const BackgroundByLayers = memo(({ styles }: SeelenWegBackgroundProps) => {
  return <div className={cs.backgroundLayers}>
    {styles.map((layer, index) => (
      <div key={index} className={cs.layer} style={layer} />
    ))}
  </div>;
});

export function SeelenWeg() {
  const theme = useSelector(Selectors.theme);
  const settings = useSelector(Selectors.settings);
  const openApps = useSelector(Selectors.apps);
  const pinnedOnLeft = useSelector(Selectors.pinnedOnLeft);
  const pinnedOnRight = useSelector(Selectors.pinnedOnRight);

  const [items, setItems] = useState<Item[]>(StoreAppsToItems(openApps, settings.size));

  const itemsContainer = useRef<HTMLDivElement>(null);

  useEffect(() => {
    setItems(StoreAppsToItems(openApps, settings.size));
  }, [openApps]);

  useEffect(() => {
    updateHitbox();
  }, [items, settings]);

  const onMouseMove = (event: MouseEvent<HTMLDivElement>) => {
    if (!itemsContainer.current) {
      return;
    }

    const mouseX = event.clientX; // Posición X del cursor

    itemsContainer.current.childNodes.forEach((child) => {
      if (child.nodeType !== Node.ELEMENT_NODE) {
        return;
      }

      const node = child as HTMLElement;
      const rect = (node as HTMLDivElement).getBoundingClientRect();
      const centerX = rect.left + rect.width / 2;

      const realDistance = Math.abs(mouseX - centerX);
      const delta = settings.zoomSize / 2 + 5;
      const distanceFromBorder = Math.max(delta, realDistance) - delta;

      const distance = Math.min(MAX_CURSOR_DISTANCE, distanceFromBorder);
      const newSize = Math.max(
        settings.size,
        ((MAX_CURSOR_DISTANCE - distance) / MAX_CURSOR_DISTANCE) * settings.zoomSize,
      );

      const distancemargin = Math.min(MAX_CURSOR_DISTANCE / 3, distanceFromBorder);
      const marginBottom =
        ((MAX_CURSOR_DISTANCE / 3 - distancemargin) / (MAX_CURSOR_DISTANCE / 3)) * 10;

      node.style.width = newSize + 'px';
      node.style.height = newSize + 'px';
      node.style.marginBottom = marginBottom + 'px';
    });
  };

  const onMouseLeave = () => {
    if (!itemsContainer.current) {
      return;
    }

    itemsContainer.current.childNodes.forEach((child) => {
      if (child.nodeType !== Node.ELEMENT_NODE) {
        return;
      }

      const node = child as HTMLElement;
      node.style.width = settings.size + 'px';
      node.style.height = settings.size + 'px';
      node.style.marginBottom = 0 + 'px';
    });
  };

  let containerStyle = {
    gap: settings.spaceBetweenItems,
    padding: settings.padding,
    height: settings.size + settings.padding * 2,
  };

  return (
    <div className={cs.bar} onMouseMoveCapture={onMouseMove} onMouseLeave={onMouseLeave}>
      <BackgroundByLayers styles={theme?.seelenweg.background || []} />
      {pinnedOnLeft.length > 0 && (
        <div className={cx(cs.itemsContainer, cs.left)} style={containerStyle}>
          {/* TODO */}
        </div>
      )}
      <div ref={itemsContainer} className={cs.itemsContainer} style={containerStyle}>
        {items.map((item, index) => (
          <Tooltip key={index} title={item.label} placement="top" showArrow={false}>
            <button
              className={cs.item}
              style={item.style}
              onClick={() => {
                invoke('weg_toggle_window_state', { hwnd: item.hwnd!, exePath: item.exePath });
              }}
            >
              <img src={convertFileSrc(item.iconPath)} />
            </button>
          </Tooltip>
        ))}
      </div>
      {pinnedOnRight.length > 0 && (
        <div className={cx(cs.itemsContainer, cs.right)} style={containerStyle}>
          {/* TODO */}
        </div>
      )}
    </div>
  );
}
