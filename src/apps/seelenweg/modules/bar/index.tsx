import {
  HideMode,
  SeelenWegItemDisplayOption,
  SeelenWegMode,
  SeelenWegSide,
  WegItemType,
} from '@seelen-ui/lib';
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

import { OpenedWindow, SeparatorWegItem, SwItem } from '../shared/store/domain';

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

function shouldBeHidden(hideMode: HideMode, isActive: boolean, isOverlaped: boolean, associatedViewCounter: number) {
  let shouldBeHidden = false;
  switch (hideMode) {
    case HideMode.Always:
      shouldBeHidden = !isActive && (associatedViewCounter == 0);
      break;
    case HideMode.Never:
      shouldBeHidden = false;
      break;
    case HideMode.OnOverlap:
      shouldBeHidden = !isActive && isOverlaped && (associatedViewCounter == 0);
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
  const monitorInfo = useSelector(Selectors.monitorInfo);

  const pinnedOnLeft = useSelector(Selectors.itemsOnLeft);
  const pinnedOnCenter = useSelector(Selectors.itemsOnCenter);
  const pinnedOnRight = useSelector(Selectors.itemsOnRight);

  const dispatch = useDispatch();
  const { t } = useTranslation();

  useWindowFocusChange((focused) => {
    if (focused)
      setAssociatedViewCounter(0);
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
    if (settings.multitaskbarItemVisibilityBehaviour != SeelenWegItemDisplayOption.AllOnAll || !monitorInfo.isPrimary) {
      return;
    }

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

  const projectSwItem = (item: SwItem) => ItemByType(item, (isOpen) => setAssociatedViewCounter((current) => calculateAssociatedViewCounter(current, isOpen)));

  let shouldBeReduced = [];
  switch (settings.multitaskbarItemVisibilityBehaviour) {
    case SeelenWegItemDisplayOption.PrimaryScreenAll:
      if (monitorInfo.isPrimary) {
        shouldBeReduced = [];
      } else {
        shouldBeReduced = [ WegItemType.Pinned, WegItemType.Temporal ];
      }
      break;
    case SeelenWegItemDisplayOption.Minimal:
      if (monitorInfo.isPrimary) {
        shouldBeReduced = [ WegItemType.Temporal ];
      } else {
        shouldBeReduced = [ WegItemType.Pinned, WegItemType.Temporal ];
      }
      break;
    default:
      shouldBeReduced = [];
      break;
  }

  function isOnMonitor(item: SwItem) {
    if (shouldBeReduced.includes(item.type)) {
      return !('opens' in item) || item.opens.some((current: OpenedWindow) => current.presentative_monitor == monitorInfo.id);
    } else {
      return true;
    }
  }

  function filterOpenElement(item: SwItem): SwItem {
    const newItem = { ...item };

    if (('opens' in item)) {
      if (monitorInfo.isPrimary) {
        newItem.opens = item.opens.filter((current: OpenedWindow) => settings.multitaskbarItemVisibilityBehaviour != SeelenWegItemDisplayOption.Minimal || current.presentative_monitor == monitorInfo.id);
      } else {
        newItem.opens = item.opens.filter((current: OpenedWindow) => current.presentative_monitor == monitorInfo.id);
      }
    }

    return newItem;
  }

  const filteredPinOnLeft = pinnedOnLeft.filter(isOnMonitor).map(filterOpenElement);
  const filteredPinOnCenter = pinnedOnCenter.filter(isOnMonitor).map(filterOpenElement);
  const filteredPinOnRight = pinnedOnRight.filter(isOnMonitor).map(filterOpenElement);

  return (
    <WithContextMenu items={getSeelenWegMenu(t)}>
      <Reorder.Group
        as="div"
        values={[...filteredPinOnLeft, Separator1, ...filteredPinOnCenter, Separator2, ...filteredPinOnRight]}
        onReorder={onReorderPinned}
        axis={isHorizontal ? 'x' : 'y'}
        className={cx('taskbar', settings.position.toLowerCase(), {
          horizontal: isHorizontal,
          vertical: !isHorizontal,
          'full-width': settings.mode === SeelenWegMode.FullWidth,
          hidden: shouldBeHidden(settings.hideMode, isActive, isOverlaped, associatedViewCounter),
          delayed,
        })}>
        <BackgroundByLayersV2 prefix="taskbar" />
        {[
          ...filteredPinOnLeft.map(projectSwItem),
          <Reorder.Item
            as="div"
            key="separator1"
            value={Separator1}
            className={cx('weg-separator weg-separator-1', {
              visible: settings.visibleSeparators,
            })}
            drag={false}
            style={getSeparatorComplementarySize(filteredPinOnLeft.length, filteredPinOnCenter.length)}
          />,
          ...filteredPinOnCenter.map(projectSwItem),
          <Reorder.Item
            as="div"
            key="separator2"
            value={Separator2}
            className={cx('weg-separator weg-separator-2', {
              visible: settings.visibleSeparators,
            })}
            drag={false}
            style={getSeparatorComplementarySize(filteredPinOnRight.length, filteredPinOnCenter.length)}
          />,
          ...filteredPinOnRight.map(projectSwItem),
        ]}
      </Reorder.Group>
    </WithContextMenu>
  );
}

function ItemByType(item: SwItem, callback: (isOpen: boolean) => void) {
  if (item.type === WegItemType.Pinned && item.path) {
    if (
      item.execution_command.startsWith('shell:AppsFolder') ||
      item.execution_command.endsWith('.exe')
    ) {
      return <UserApplication key={item.execution_command} item={item} onAssociatedViewOpenChanged={callback} />;
    }
    return <FileOrFolder key={item.execution_command} item={item} />;
  }

  if (item.type === WegItemType.Temporal && item.path) {
    return <UserApplication key={item.execution_command} item={item} onAssociatedViewOpenChanged={callback} />;
  }

  if (item.type === WegItemType.Media) {
    return <MediaSession key="media-item" item={item} />;
  }

  if (item.type === WegItemType.StartMenu) {
    return <StartMenu key="start-menu" item={item} />;
  }

  return null;
}
