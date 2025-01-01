import { HideMode, SeelenWegMode, SeelenWegSide, WegItemType } from '@seelen-ui/lib';
import { Reorder } from 'framer-motion';
import { useCallback, useLayoutEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useDispatch, useSelector } from 'react-redux';

import { BackgroundByLayersV2 } from '../../components/BackgroundByLayers/infra';
import { FileOrFolder } from '../item/infra/File';
import { MediaSession } from '../item/infra/MediaSession';
import { StartMenu } from '../item/infra/StartMenu';
import { UserApplication } from '../item/infra/UserApplication';

import { RootActions, Selectors } from '../shared/store/app';
import { useWindowFocusChange } from 'src/apps/shared/hooks';

import { SeparatorWegItem, SwItem } from '../shared/store/domain';

import { cx } from '../../../shared/styles';
import { WithContextMenu } from '../../components/WithContextMenu';
import { savePinnedItems } from '../shared/store/storeApi';
import { getSeelenWegMenu } from './menu';

const Separator1: SeparatorWegItem = {
  id: '1',
  type: WegItemType.Separator,
};

const Separator2: SeparatorWegItem = {
  id: '2',
  type: WegItemType.Separator,
};

function shouldBeHidden(
  hideMode: HideMode,
  isActive: boolean,
  isOverlaped: boolean,
  associatedViewCounter: number,
) {
  let shouldBeHidden = false;
  switch (hideMode) {
    case HideMode.Always:
      shouldBeHidden = !isActive && associatedViewCounter == 0;
      break;
    case HideMode.Never:
      shouldBeHidden = false;
      break;
    case HideMode.OnOverlap:
      shouldBeHidden = !isActive && isOverlaped && associatedViewCounter == 0;
  }
  return shouldBeHidden;
}

function calculateAssociatedViewCounter(currentValue: number, currentChange: boolean): number {
  const newValue = currentValue + (currentChange ? 1 : -1);
  return newValue >= 0 ? newValue : currentValue;
}

export function SeelenWeg() {
  const [isActive, setActive] = useState(false);
  const [delayed, setDelayed] = useState(false);
  // Counts every associated window in the bar and will act as a reverse mutex for the hide functionality
  const [associatedViewCounter, setAssociatedViewCounter] = useState(0);

  const settings = useSelector(Selectors.settings);
  const isOverlaped = useSelector(Selectors.isOverlaped);

  const pinnedOnLeft = useSelector(Selectors.itemsOnLeft);
  const pinnedOnCenter = useSelector(Selectors.itemsOnCenter);
  const pinnedOnRight = useSelector(Selectors.itemsOnRight);

  const dispatch = useDispatch();
  const { t } = useTranslation();

  useWindowFocusChange((focused) => {
    if (focused) setAssociatedViewCounter(0);
    setActive(focused);
  });

  useLayoutEffect(() => {
    switch (settings.hideMode) {
      case HideMode.Always:
        setDelayed(true);
        break;
      case HideMode.Never:
        setDelayed(false);
        break;
      case HideMode.OnOverlap:
        if (!isOverlaped) {
          setDelayed(false);
          break;
        }
        setTimeout(() => {
          setDelayed(true);
        }, 300);
        break;
    }
  }, [isOverlaped, settings]);

  const getSeparatorComplementarySize = useCallback(
    (sideElements: number, centerElements: number) => {
      let size = '1px';

      if (settings.mode === SeelenWegMode.FullWidth) {
        size = `calc(50% - (${settings.size + settings.spaceBetweenItems}px * ${
          sideElements + centerElements / 2
        }) - ${settings.spaceBetweenItems}px)`;
      }

      if (settings.position === SeelenWegSide.Top || settings.position === SeelenWegSide.Bottom) {
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

  const onReorderPinned = useCallback((apps: SwItem[]) => {
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

      if (app.type !== WegItemType.Separator) {
        extractedPinned.push(app);
      }
    });

    dispatch(RootActions.setItemsOnRight(extractedPinned));
    savePinnedItems();
  }, []);

  const isHorizontal =
    settings.position === SeelenWegSide.Top || settings.position === SeelenWegSide.Bottom;

  const shit = useCallback((isOpen: boolean) => {
    setAssociatedViewCounter((current) => calculateAssociatedViewCounter(current, isOpen));
  }, []);

  const projectSwItem = (item: SwItem) => ItemByType(item, isHorizontal ? 'x' : 'y', shit);

  console.log(
    'pinnedOnLeft',
    pinnedOnLeft,
    'pinnedOnCenter',
    pinnedOnCenter,
    'pinnedOnRight',
    pinnedOnRight,
  );

  return (
    <WithContextMenu items={getSeelenWegMenu(t)}>
      <Reorder.Group
        as="div"
        values={[...pinnedOnLeft, Separator1, ...pinnedOnCenter, Separator2, ...pinnedOnRight]}
        onReorder={onReorderPinned}
        axis={isHorizontal ? 'x' : 'y'}
        className={cx('taskbar', settings.position.toLowerCase(), {
          horizontal: isHorizontal,
          vertical: !isHorizontal,
          'full-width': settings.mode === SeelenWegMode.FullWidth,
          hidden: shouldBeHidden(settings.hideMode, isActive, isOverlaped, associatedViewCounter),
          delayed,
        })}
      >
        <BackgroundByLayersV2 prefix="taskbar" />
        <div className="taskbar-scroll-container" onWheel={scrollXonY}>
          <div className="taskbar-scroll-inner-frame">
            {[
              ...pinnedOnLeft.map(projectSwItem),
              <Reorder.Item
                as="div"
                key="separator1"
                value={Separator1}
                className={cx('weg-separator weg-separator-1', {
                  visible: settings.visibleSeparators,
                })}
                drag={false}
                style={getSeparatorComplementarySize(pinnedOnLeft.length, pinnedOnCenter.length)}
              />,
              ...pinnedOnCenter.map(projectSwItem),
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
              ...pinnedOnRight.map(projectSwItem),
            ]}
          </div>
        </div>
      </Reorder.Group>
    </WithContextMenu>
  );
}

function scrollXonY(e) {
  e.preventDefault();

  // Capture up/down wheel events and scroll the viewport horizontally
  const delta = e.deltaY;
  const currPos = e.currentTarget.scrollLeft;
  const scrollWidth = e.currentTarget.scrollWidth;

  const newPos = Math.max(0, Math.min(scrollWidth, currPos + delta / 5));

  e.currentTarget.scrollLeft = newPos;
}

function ItemByType(item: SwItem, drag: boolean | 'x' | 'y' | undefined, callback: (isOpen: boolean) => void) {
  if (item.type === WegItemType.Pinned && item.path) {
    if (
      item.path.toLowerCase().endsWith('.exe') ||
      item.relaunchCommand.toLowerCase().includes('.exe')
    ) {
      return <UserApplication drag={drag} key={item.id} item={item} onAssociatedViewOpenChanged={callback} />;
    }
    return <FileOrFolder drag={drag} key={item.id} item={item} />;
  }

  if (item.type === WegItemType.Temporal) {
    return <UserApplication drag={drag} key={item.id} item={item} onAssociatedViewOpenChanged={callback} />;
  }

  if (item.type === WegItemType.Media) {
    return <MediaSession drag={drag} key={item.id} item={item} />;
  }

  if (item.type === WegItemType.StartMenu) {
    return <StartMenu key={item.id} item={item} />;
  }

  return null;
}
