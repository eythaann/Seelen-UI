import { debounce } from '../../../Timing';
import { ExtraCallbacksOnLeave } from '../../events';
import { WegItem } from './item';
import { Reorder } from 'framer-motion';
import { MouseEvent, useCallback, useEffect, useRef } from 'react';
import { useDispatch, useSelector } from 'react-redux';

import { BackgroundByLayers } from '../../components/BackgrounByLayers/infra';

import { RootActions, Selectors } from '../shared/store/app';

import { SeelenWegMode } from '../../../settings/modules/seelenweg/domain';
import { App, Separator, SpecialItemType } from '../shared/store/domain';

const MAX_CURSOR_DISTANCE = 500;
const MAX_CURSOR_DISTANCE_MARGIN = MAX_CURSOR_DISTANCE / 3;

const Separator1: Separator = {
  type: SpecialItemType.Separator,
};

const Separator2: Separator = {
  type: SpecialItemType.Separator,
};

export function SeelenWeg() {
  const focusedHandle = useSelector(Selectors.focusedHandle);
  const theme = useSelector(Selectors.theme);
  const settings = useSelector(Selectors.settings);

  const pinnedOnLeft = useSelector(Selectors.pinnedOnLeft);
  const pinnedOnCenter = useSelector(Selectors.pinnedOnCenter);
  const pinnedOnRight = useSelector(Selectors.pinnedOnRight);

  const refs = useRef<HTMLDivElement[]>([]);
  const separatorRefs = useRef<HTMLDivElement[]>([]);
  const lenghtsRefs = useRef<number[]>([]);

  const shouldAnimate = useRef(false);
  const mouseX = useRef(0);
  const timeoutRef = useRef<ReturnType<typeof setTimeout>>(null);

  const dispatch = useDispatch();

  useEffect(() => {
    ExtraCallbacksOnLeave.add(() => {
      shouldAnimate.current = false;
    });
  }, []);

  useEffect(() => {
    refs.current = Array.from(document.getElementsByClassName('weg-item')) as HTMLDivElement[];
    separatorRefs.current = Array.from(
      document.getElementsByClassName('weg-separator'),
    ) as HTMLDivElement[];
    lenghtsRefs.current = [pinnedOnLeft.length, pinnedOnCenter.length, pinnedOnRight.length];
  });

  const animate = useCallback(() => {
    let totalLeftSize = 0;
    let totalCenterSize = 0;
    let totalRightSize = 0;

    if (!shouldAnimate.current) {
      refs.current.forEach((child) => {
        const node = child as HTMLElement;
        node.style.width = settings.size + 'px';
        node.style.height = settings.size + 'px';
        node.style.marginBottom = 0 + 'px';
      });

      totalLeftSize = (settings.size + settings.spaceBetweenItems) * lenghtsRefs.current[0]!;
      totalCenterSize = (settings.size + settings.spaceBetweenItems) * lenghtsRefs.current[1]!;
      totalRightSize = (settings.size + settings.spaceBetweenItems) * lenghtsRefs.current[2]!;

      separatorRefs.current[0]!.style.width = `calc(50% - ${totalLeftSize + totalCenterSize / 2}px`;
      separatorRefs.current[1]!.style.width = `calc(50% - ${
        totalRightSize + totalCenterSize / 2
      }px`;
      return;
    }

    const stop1 = lenghtsRefs.current[0]!;
    const stop2 = stop1 + lenghtsRefs.current[1]!;
    const stop3 = stop2 + lenghtsRefs.current[2]!;
    refs.current.forEach((child, index) => {
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

      if (index < stop1) {
        totalLeftSize += newSize + settings.spaceBetweenItems;
      } else if (index < stop2) {
        totalCenterSize += newSize + settings.spaceBetweenItems;
      } else if (index < stop3) {
        totalRightSize += newSize + settings.spaceBetweenItems;
      }

      node.style.width = newSize + 'px';
      node.style.height = newSize + 'px';
      node.style.marginBottom = marginBottom + 'px';
    });

    separatorRefs.current[0]!.style.width = `calc(50% - ${totalLeftSize + totalCenterSize / 2}px`;
    separatorRefs.current[1]!.style.width = `calc(50% - ${totalRightSize + totalCenterSize / 2}px`;

    requestAnimationFrame(animate);
  }, [settings]);

  const onMouseMove = useCallback((event: MouseEvent<HTMLDivElement>) => {
    event.stopPropagation();
    mouseX.current = event.clientX;
  }, []);

  const onMouseEnter = useCallback(() => {
    shouldAnimate.current = true;
    requestAnimationFrame(animate);
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current);
    }
  }, [settings]);

  const disableMouseAnimations = useCallback(() => {
    shouldAnimate.current = false;
  }, []);

  const onReorderPinneds = useCallback((apps: (Separator | App)[]) => {
    let extractedPinned: App[] = [];

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

      if (app.type !== SpecialItemType.Separator) {
        extractedPinned.push(app);
      }
    });

    dispatch(RootActions.setPinnedOnRight(extractedPinned));
  }, []);

  return (
    <Reorder.Group
      onMouseEnter={onMouseEnter}
      onMouseMove={onMouseMove}
      onMouseLeave={debounce(disableMouseAnimations, 100, timeoutRef)}
      values={[...pinnedOnLeft, Separator1, ...pinnedOnCenter, Separator2, ...pinnedOnRight]}
      onReorder={onReorderPinneds}
      axis="x"
      as="div"
      className="taskbar"
    >
      <BackgroundByLayers prefix="taskbar" styles={theme?.seelenweg.backgroundLayers || []} />
      <div className={'weg-items-container'}>
        {[
          ...pinnedOnLeft.map((item) => (
            <WegItem
              key={item.exe}
              item={item}
              initialSize={settings.size}
              isFocused={item.opens.includes(focusedHandle)}
            />
          )),
          <Reorder.Item
            as="div"
            key="separator1"
            value={Separator1}
            className={'weg-separator'}
            onDragStart={(e) => e.stopPropagation()}
            style={{
              height: settings.size,
              marginLeft: pinnedOnLeft.length ? 0 : settings.spaceBetweenItems * -1,
              width:
                settings.mode === SeelenWegMode.FULL_WIDTH
                  ? `calc(50% - ${settings.size + settings.spaceBetweenItems}px * ${
                    pinnedOnLeft.length + pinnedOnCenter.length / 2
                  })`
                  : 'auto',
            }}
          />,
          ...pinnedOnCenter.map((item) => (
            <WegItem
              key={item.exe}
              item={item}
              initialSize={settings.size}
              isFocused={item.opens.includes(focusedHandle)}
            />
          )),
          <Reorder.Item
            as="div"
            key="separator2"
            value={Separator2}
            className={'weg-separator'}
            style={{
              height: settings.size,
              marginLeft: pinnedOnRight.length ? 0 : settings.spaceBetweenItems * -1,
              width:
                settings.mode === SeelenWegMode.FULL_WIDTH
                  ? `calc(50% - ${settings.size + settings.spaceBetweenItems}px * ${
                    pinnedOnRight.length + pinnedOnCenter.length / 2
                  })`
                  : 'auto',
            }}
          />,
          ...pinnedOnRight.map((item) => (
            <WegItem
              key={item.exe}
              item={item}
              initialSize={settings.size}
              isFocused={item.opens.includes(focusedHandle)}
            />
          )),
        ]}
      </div>
    </Reorder.Group>
  );
}
