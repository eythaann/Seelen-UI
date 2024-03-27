import { WegItem } from './item';
import { Reorder } from 'framer-motion';
import { debounce } from 'lodash';
import { MouseEvent, useEffect, useRef, useState } from 'react';
import { useDispatch, useSelector } from 'react-redux';

import { BackgroundByLayers } from '../../components/BackgrounByLayers/infra';
import cs from './infra.module.css';

import { cx } from '../../../settings/modules/shared/app/utils';
import { RootActions, Selectors } from '../shared/store/app';

import { App, OpenApp, PinnedApp, SpecialApp, SpecialItemType } from '../shared/store/domain';

const MAX_CURSOR_DISTANCE = 500;
const MAX_CURSOR_DISTANCE_MARGIN = MAX_CURSOR_DISTANCE / 3;

const Separator1: SpecialApp = {
  type: SpecialItemType.Separator,
};
const Separator2: SpecialApp = {
  type: SpecialItemType.Separator,
};
const Separator3: SpecialApp = {
  type: SpecialItemType.Separator,
};

export function SeelenWeg() {
  const theme = useSelector(Selectors.theme);
  const settings = useSelector(Selectors.settings);

  const openApps = useSelector(Selectors.apps);
  const pinnedOnLeft = useSelector(Selectors.pinnedOnLeft);
  const pinnedOnCenter = useSelector(Selectors.pinnedOnCenter);
  const pinnedOnRight = useSelector(Selectors.pinnedOnRight);

  const refs = useRef<HTMLDivElement[]>([]);

  const dispatch = useDispatch();

  useEffect(() => {
    refs.current = Array.from(document.getElementsByClassName(cs.item!)) as HTMLDivElement[];
  });

  const onReorderPinneds = debounce((apps: (SpecialApp | PinnedApp)[]) => {
    console.log(apps);
    let extractedPinned: PinnedApp[] = [];

    apps.forEach((app) => {
      if (app === Separator1) {
        dispatch(RootActions.setPinnedOnLeft(extractedPinned));
        extractedPinned = [];
        return;
      }

      if (app === Separator2) {
        dispatch(RootActions.setPinnedOnCenter(extractedPinned));
        extractedPinned = [];
        return;
      }

      extractedPinned.push(app as PinnedApp);
    });

    dispatch(RootActions.setPinnedOnRight(extractedPinned));
  }, 200);

  const onReorderOpens = (apps: OpenApp[]) => {
    dispatch(RootActions.setApps(apps));
  };

  const onMouseMove = (event: MouseEvent<HTMLDivElement>) => {
    const mouseX = event.clientX; // Posición X del cursor

    refs.current.forEach((child) => {
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

      const maxMargin = (settings.zoomSize - settings.size) / 5;
      const distancemargin = Math.min(MAX_CURSOR_DISTANCE_MARGIN, distanceFromBorder);
      const marginBottom =
        ((MAX_CURSOR_DISTANCE_MARGIN - distancemargin) / MAX_CURSOR_DISTANCE_MARGIN) * maxMargin;

      node.style.width = newSize + 'px';
      node.style.height = newSize + 'px';
      node.style.marginBottom = marginBottom + 'px';
    });
  };

  const onMouseLeave = () => {
    refs.current.forEach((child) => {
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
    <Reorder.Group
      onMouseMoveCapture={onMouseMove}
      onMouseLeave={onMouseLeave}
      values={[...pinnedOnLeft, Separator1, ...pinnedOnCenter, Separator2, ...pinnedOnRight]}
      onReorder={onReorderPinneds}
      axis="x"
      as="div"
      className={cs.bar}
    >
      <BackgroundByLayers styles={theme?.seelenweg.background || []} />

      <div
        className={cx(cs.itemsContainer, cs.left)}
        style={{
          ...containerStyle,
          padding: pinnedOnLeft.length === 0 ? 0 : containerStyle.padding,
        }}
      >
        {pinnedOnLeft.map((item) => (
          <WegItem key={item.exe} item={item} initialSize={settings.size} />
        ))}
      </div>

      <Reorder.Item as="div" value={Separator1}>
        <div>|</div>
      </Reorder.Item>

      <div className={cs.group}>
        <div
          className={cs.itemsContainer}
          style={containerStyle}
        >
          {pinnedOnCenter.map((item) => (
            <WegItem key={item.exe} item={item} initialSize={settings.size} />
          ))}
        </div>

        <Reorder.Group
          as="div"
          axis="x"
          className={cs.itemsContainer}
          style={containerStyle}
          values={openApps}
          onReorder={onReorderOpens}
        >
          {openApps.map((item) => (
            <WegItem key={item.hwnd} item={item} initialSize={settings.size} />
          ))}
        </Reorder.Group>
      </div>

      <Reorder.Item as="div" value={Separator2}>
        <div>|</div>
      </Reorder.Item>

      <div
        className={cx(cs.itemsContainer, cs.right)}
        style={{
          ...containerStyle,
          padding: pinnedOnRight.length === 0 ? 0 : containerStyle.padding,
        }}
      >
        {pinnedOnRight.map((item) => (
          <WegItem key={item.exe} item={item} initialSize={settings.size} />
        ))}
      </div>
    </Reorder.Group>
  );
}
