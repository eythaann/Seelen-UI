import { SeelenWegHideMode, SeelenWegMode, SeelenWegSide } from '../../../shared/schemas/Seelenweg';
import { SavedSeparatorItem } from '../../../shared/schemas/SeelenWegItems';
import { cx } from '../../../shared/styles';
import { WegItem } from './item';
import { Reorder } from 'framer-motion';
import { MouseEvent, useCallback, useEffect, useRef, useState } from 'react';
import { useDispatch, useSelector } from 'react-redux';

import { BackgroundByLayers } from '../../components/BackgrounByLayers/infra';
import { useAppActivation, useAppBlur } from '../shared/hooks/infra';

import { RootActions, Selectors } from '../shared/store/app';

import { SpecialItemType, SwItem } from '../shared/store/domain';

const MAX_CURSOR_DISTANCE = window.screen.height / 3;
const MAX_CURSOR_DISTANCE_MARGIN = MAX_CURSOR_DISTANCE / 3;

const Separator1: SavedSeparatorItem = {
  type: SpecialItemType.Separator,
};

const Separator2: SavedSeparatorItem = {
  type: SpecialItemType.Separator,
};

function shouldBeHidden(hideMode: SeelenWegHideMode, isActive: boolean, isOverlaped: boolean) {
  let shouldBeHidden = false;
  switch (hideMode) {
    case SeelenWegHideMode.Always:
      shouldBeHidden = !isActive;
      break;
    case SeelenWegHideMode.Never:
      shouldBeHidden = false;
      break;
    case SeelenWegHideMode.OnOverlap:
      shouldBeHidden = !isActive && isOverlaped;
  }
  return shouldBeHidden;
}

export function SeelenWeg() {
  const focusedHandle = useSelector(Selectors.focusedHandle);
  const bgLayers = useSelector(Selectors.themeLayers);
  const settings = useSelector(Selectors.settings);
  const isOverlaped = useSelector(Selectors.isOverlaped);

  const pinnedOnLeft = useSelector(Selectors.itemsOnLeft);
  const pinnedOnCenter = useSelector(Selectors.itemsOnCenter);
  const pinnedOnRight = useSelector(Selectors.itemsOnRight);

  const [isActive, setActive] = useState(false);

  const refs = useRef<HTMLDivElement[]>([]);
  const separatorRefs = useRef<HTMLDivElement[]>([]);
  const lengthsRefs = useRef<number[]>([]);

  const shouldAnimate = useRef(false);
  const mousePos = useRef({
    x: 0,
    y: 0,
  });

  const dispatch = useDispatch();

  useAppBlur(() => {
    shouldAnimate.current = false;
    setActive(false);
  }, [settings]);

  useAppActivation(() => {
    shouldAnimate.current = true;
    requestAnimationFrame(animate);
    setActive(true);
  }, [settings]);

  useEffect(() => {
    refs.current = Array.from(document.getElementsByClassName('weg-item')) as HTMLDivElement[];
    separatorRefs.current = Array.from(
      document.getElementsByClassName('weg-separator'),
    ) as HTMLDivElement[];
    lengthsRefs.current = [pinnedOnLeft.length, pinnedOnCenter.length, pinnedOnRight.length];
  });

  const getSeparatorComplementarySize = useCallback(
    (sideElements: number, centerElements: number) => {
      let size = '1px';

      if (settings.mode === SeelenWegMode.FULL_WIDTH) {
        size = `calc(50% - (${settings.size + settings.spaceBetweenItems}px * ${
          sideElements + centerElements / 2
        }) - ${settings.spaceBetweenItems}px)`;
      }

      if (settings.position === SeelenWegSide.TOP || settings.position === SeelenWegSide.BOTTOM) {
        return {
          width: size,
        };
      }

      return {
        height: size,
      };
    },
    [settings],
  );

  const animate = useCallback(() => {
    let totalLeftSize = 0;
    let totalCenterSize = 0;
    let totalRightSize = 0;
    const isFullWidth = settings.mode === SeelenWegMode.FULL_WIDTH;
    const isHorizontal =
      settings.position === SeelenWegSide.TOP || settings.position === SeelenWegSide.BOTTOM;

    if (!shouldAnimate.current) {
      refs.current.forEach((child) => {
        const node = child as HTMLElement;
        node.style.width = settings.size + 'px';
        node.style.height = settings.size + 'px';
        node.style[`margin${settings.position}`] = 0 + 'px';
      });

      if (isFullWidth) {
        const complementarySize1 = getSeparatorComplementarySize(
          lengthsRefs.current[0]!,
          lengthsRefs.current[1]!,
        );
        const complementarySize2 = getSeparatorComplementarySize(
          lengthsRefs.current[2]!,
          lengthsRefs.current[1]!,
        );

        if (isHorizontal) {
          separatorRefs.current[0]!.style.width = complementarySize1.width!;
          separatorRefs.current[1]!.style.width = complementarySize2.width!;
        } else {
          separatorRefs.current[0]!.style.height = complementarySize1.height!;
          separatorRefs.current[1]!.style.height = complementarySize2.height!;
        }
      }

      return;
    }

    const stop1 = lengthsRefs.current[0]!;
    const stop2 = stop1 + lengthsRefs.current[1]!;
    const stop3 = stop2 + lengthsRefs.current[2]!;
    refs.current.forEach((child, index) => {
      const node = child as HTMLElement;
      const rect = (node as HTMLDivElement).getBoundingClientRect();
      const centerX = rect.left + rect.width / 2;
      const centerY = rect.top + rect.height / 2;

      const realDistance = isHorizontal
        ? Math.abs(mousePos.current.x - centerX)
        : Math.abs(mousePos.current.y - centerY);
      const delta = settings.zoomSize / 2 + 5;
      const distanceFromBorder = Math.max(delta, realDistance) - delta;

      const distance = Math.min(MAX_CURSOR_DISTANCE, distanceFromBorder);
      const newSize = Math.max(
        settings.size,
        ((MAX_CURSOR_DISTANCE - distance) / MAX_CURSOR_DISTANCE) * settings.zoomSize,
      );

      const maxMargin = (settings.zoomSize - settings.size) / 5;
      const distanceMargin = Math.min(MAX_CURSOR_DISTANCE_MARGIN, distanceFromBorder);
      const marginSize =
        ((MAX_CURSOR_DISTANCE_MARGIN - distanceMargin) / MAX_CURSOR_DISTANCE_MARGIN) * maxMargin;

      node.style.width = newSize + 'px';
      node.style.height = newSize + 'px';
      node.style[`margin${settings.position}`] = marginSize + 'px';

      if (!isFullWidth) {
        return;
      }

      if (index < stop1) {
        totalLeftSize += newSize + settings.spaceBetweenItems;
      } else if (index < stop2) {
        totalCenterSize += newSize + settings.spaceBetweenItems;
      } else if (index < stop3) {
        totalRightSize += newSize + settings.spaceBetweenItems;
      }
    });

    if (isFullWidth) {
      const complementarySize1 = `calc(50% - ${totalLeftSize + totalCenterSize / 2}px - ${
        settings.spaceBetweenItems
      }px`;
      const complementarySize2 = `calc(50% - ${totalRightSize + totalCenterSize / 2}px - ${
        settings.spaceBetweenItems
      }px`;

      if (isHorizontal) {
        separatorRefs.current[0]!.style.width = complementarySize1;
        separatorRefs.current[1]!.style.width = complementarySize2;
      } else {
        separatorRefs.current[0]!.style.height = complementarySize1;
        separatorRefs.current[1]!.style.height = complementarySize2;
      }
    }

    requestAnimationFrame(animate);
  }, [settings]);

  const onMouseMove = useCallback((event: MouseEvent<HTMLDivElement>) => {
    event.stopPropagation();
    mousePos.current.x = event.clientX;
    mousePos.current.y = event.clientY;
  }, []);

  const onReorderPinned = useCallback((apps: (SavedSeparatorItem | SwItem)[]) => {
    let extractedPinned: SwItem[] = [];

    apps.forEach((app) => {
      if (app === Separator1) {
        dispatch(RootActions.setItemsOnLeft(extractedPinned));
        extractedPinned = [];
        return;
      }

      if (app === Separator2) {
        dispatch(RootActions.setItemsOnCenter(extractedPinned));
        extractedPinned = [];
        return;
      }

      if (app.type !== SpecialItemType.Separator) {
        extractedPinned.push(app);
      }
    });

    dispatch(RootActions.setItemsOnRight(extractedPinned));
  }, []);

  const isHorizontal =
    settings.position === SeelenWegSide.TOP || settings.position === SeelenWegSide.BOTTOM;

  return (
    <Reorder.Group
      onMouseMove={onMouseMove}
      values={[...pinnedOnLeft, Separator1, ...pinnedOnCenter, Separator2, ...pinnedOnRight]}
      onReorder={onReorderPinned}
      axis={isHorizontal ? 'x' : 'y'}
      as="div"
      className={cx('taskbar', settings.position.toLowerCase(), {
        horizontal: isHorizontal,
        vertical: !isHorizontal,
        'full-width': settings.mode === SeelenWegMode.FULL_WIDTH,
        hidden: shouldBeHidden(settings.hideMode, isActive, isOverlaped),
      })}
    >
      <BackgroundByLayers prefix="taskbar" layers={bgLayers.weg.bg || []} />
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
          className={cx('weg-separator weg-separator-1', {
            visible: settings.visibleSeparators,
          })}
          onDragStart={(e) => e.stopPropagation()}
          drag={false}
          style={getSeparatorComplementarySize(pinnedOnLeft.length, pinnedOnCenter.length)}
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
          className={cx('weg-separator weg-separator-2', {
            visible: settings.visibleSeparators,
          })}
          drag={false}
          style={getSeparatorComplementarySize(pinnedOnRight.length, pinnedOnCenter.length)}
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
    </Reorder.Group>
  );
}
