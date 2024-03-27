import { WegItem } from './item';
import { Reorder } from 'framer-motion';
import { MouseEvent, useEffect, useRef } from 'react';
import { useDispatch, useSelector } from 'react-redux';

import { BackgroundByLayers } from '../../components/BackgrounByLayers/infra';
import cs from './infra.module.css';

import { cx } from '../../../settings/modules/shared/app/utils';
import { RootActions, Selectors } from '../shared/store/app';

import { OpenApp, PinnedApp, SpecialApp, SpecialItemType } from '../shared/store/domain';

const MAX_CURSOR_DISTANCE = 500;
const MAX_CURSOR_DISTANCE_MARGIN = MAX_CURSOR_DISTANCE / 3;

const Separator1: SpecialApp = {
  type: SpecialItemType.Separator,
};
const Separator2: SpecialApp = {
  type: SpecialItemType.Separator,
};

interface Props {
  items: PinnedApp[];
  openApps?: OpenApp[];
  initialSize: number;
  align?: 'left' | 'right';
}
function ItemsContainer({ items, initialSize, align, openApps }: Props) {
  const dispatch = useDispatch();

  const onReorderOpens = (apps: OpenApp[]) => {
    dispatch(RootActions.setApps(apps));
  };

  const alignClassname = align ? cs[align] : '';
  return (
    <div className={cx(cs.itemsContainer, alignClassname)}>
      {items.map((item) => (
        <WegItem key={item.exe} item={item} initialSize={initialSize} />
      ))}
      {!!openApps && (
        <Reorder.Group
          as="div"
          axis="x"
          className={cs.itemsContainer}
          values={openApps}
          onReorder={onReorderOpens}
        >
          {openApps.map((item) => (
            <WegItem key={item.hwnd} item={item} initialSize={initialSize} />
          ))}
        </Reorder.Group>
      )}
    </div>
  );
}

export function SeelenWeg() {
  const theme = useSelector(Selectors.theme);
  const settings = useSelector(Selectors.settings);

  const openApps = useSelector(Selectors.apps);
  const pinnedOnLeft = useSelector(Selectors.pinnedOnLeft);
  const pinnedOnCenter = useSelector(Selectors.pinnedOnCenter);
  const pinnedOnRight = useSelector(Selectors.pinnedOnRight);

  const refs = useRef<HTMLDivElement[]>([]);
  const shouldAnimate = useRef(false);
  const mouseX = useRef(0);

  const dispatch = useDispatch();

  useEffect(() => {
    refs.current = Array.from(document.getElementsByClassName(cs.item!)) as HTMLDivElement[];
  });

  const animate = () => {
    refs.current.forEach((child) => {
      const node = child as HTMLElement;
      const rect = (node as HTMLDivElement).getBoundingClientRect();
      const centerX = rect.left + rect.width / 2;

      const realDistance = Math.abs(mouseX.current - centerX);
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

    if (shouldAnimate.current) {
      requestAnimationFrame(animate);
    }
  };

  const onReorderPinneds = (apps: (SpecialApp | PinnedApp)[]) => {
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
  };

  const onMouseMove = (event: MouseEvent<HTMLDivElement>) => {
    mouseX.current = event.clientX;
  };

  const onMouseEnter = () => {
    shouldAnimate.current = true;
    requestAnimationFrame(animate);
  };

  const onMouseLeave = () => {
    shouldAnimate.current = false;
    // wait for next frame before leave
    setTimeout(() => {
      refs.current.forEach((child) => {
        const node = child as HTMLElement;
        node.style.width = settings.size + 'px';
        node.style.height = settings.size + 'px';
        node.style.marginBottom = 0 + 'px';
      });
    }, 100);
  };

  return (
    <Reorder.Group
      onMouseEnter={onMouseEnter}
      onMouseMoveCapture={onMouseMove}
      onMouseLeave={onMouseLeave}
      values={[...pinnedOnLeft, Separator1, ...pinnedOnCenter, Separator2, ...pinnedOnRight]}
      onReorder={onReorderPinneds}
      axis="x"
      as="div"
      className={cs.bar}
      style={{
        padding: settings.padding,
        height: settings.size + settings.padding * 2,
        gap: settings.spaceBetweenItems,
      }}
    >
      <BackgroundByLayers styles={theme?.seelenweg.background || []} />
      <ItemsContainer items={pinnedOnLeft} align="left" initialSize={settings.size} />
      <Reorder.Item
        as="div"
        value={Separator1}
        className={cs.separator}
        style={{ height: settings.size }}
      />
      <ItemsContainer items={pinnedOnCenter} openApps={openApps} initialSize={settings.size} />
      <Reorder.Item
        as="div"
        value={Separator2}
        className={cs.separator}
        style={{ height: settings.size }}
      />
      <ItemsContainer items={pinnedOnRight} align="right" initialSize={settings.size} />
    </Reorder.Group>
  );
}
