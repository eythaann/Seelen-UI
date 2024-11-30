import { Reorder } from 'framer-motion';
import { useCallback, useLayoutEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useDispatch, useSelector } from 'react-redux';
import {
  HideMode,
  MonitorInfo,
  SeelenWegItemDisplayOption,
  SeelenWegMode,
  SeelenWegSide,
  SeparatorWegItem,
  SwItemType,
  useWindowFocusChange,
} from 'seelen-core';

import { BackgroundByLayersV2 } from '../../components/BackgroundByLayers/infra';
import { FileOrFolder } from '../item/infra/File';
import { MediaSession } from '../item/infra/MediaSession';
import { StartMenu } from '../item/infra/StartMenu';
import { UserApplication } from '../item/infra/UserApplication';

import { RootActions, Selectors } from '../shared/store/app';

import { OpenedWindow, SwItem } from '../shared/store/domain';

import { cx } from '../../../shared/styles';
import { WithContextMenu } from '../../components/WithContextMenu';
import { savePinnedItems } from '../shared/store/storeApi';
import { getSeelenWegMenu } from './menu';

const Separator1: SeparatorWegItem = {
  id: '1',
  type: SwItemType.Separator,
};

const Separator2: SeparatorWegItem = {
  id: '2',
  type: SwItemType.Separator,
};

function shouldBeHidden(hideMode: HideMode, isActive: boolean, isOverlaped: boolean) {
  let shouldBeHidden = false;
  switch (hideMode) {
    case HideMode.Always:
      shouldBeHidden = !isActive;
      break;
    case HideMode.Never:
      shouldBeHidden = false;
      break;
    case HideMode.OnOverlap:
      shouldBeHidden = !isActive && isOverlaped;
  }
  return shouldBeHidden;
}

export function SeelenWeg() {
  const [isActive, setActive] = useState(false);
  const [delayed, setDelayed] = useState(false);

  const settings = useSelector(Selectors.settings);
  const isOverlaped = useSelector(Selectors.isOverlaped);

  const pinnedOnLeft = useSelector(Selectors.itemsOnLeft);
  const pinnedOnCenter = useSelector(Selectors.itemsOnCenter);
  const pinnedOnRight = useSelector(Selectors.itemsOnRight);

  const monitorInfo: MonitorInfo = useSelector(Selectors.monitorInfo);

  const dispatch = useDispatch();
  const { t } = useTranslation();

  useWindowFocusChange((focused) => {
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

      if (app.type !== SwItemType.Separator) {
        extractedPinned.push(app);
      }
    });

    dispatch(RootActions.setItemsOnRight(extractedPinned));
    savePinnedItems();
  }, []);

  const isHorizontal =
    settings.position === SeelenWegSide.Top || settings.position === SeelenWegSide.Bottom;

  let shouldBeReduced = [];
  switch (settings.multitaskbarItemVisibilityBehaviour) {
    case SeelenWegItemDisplayOption.PrimaryScreenAll:
      if (monitorInfo.isPrimary) {
        shouldBeReduced = [];
      } else {
        shouldBeReduced = [ SwItemType.Pinned, SwItemType.TemporalApp ];
      }
      break;
    case SeelenWegItemDisplayOption.Minimal:
      if (monitorInfo.isPrimary) {
        shouldBeReduced = [ SwItemType.TemporalApp ];
      } else {
        shouldBeReduced = [ SwItemType.Pinned, SwItemType.TemporalApp ];
      }
      break;
    default:
      shouldBeReduced = [];
      break;
  }

  function isOnMonitor(item: SwItem) {
    if (shouldBeReduced.includes(item.type)) {
      return !('opens' in item) || item.opens.some((current: OpenedWindow) => current.presentative_monitor == monitorInfo.index);
    } else {
      return true;
    }
  }

  function filterOpenElement(item: SwItem): SwItem {
    const newItem = { ...item };

    if (('opens' in item)) {
      if (monitorInfo.isPrimary) {
        newItem.opens = item.opens.filter((current: OpenedWindow) => settings.multitaskbarItemVisibilityBehaviour != SeelenWegItemDisplayOption.Minimal || current.presentative_monitor == monitorInfo.index);
      } else {
        newItem.opens = item.opens.filter((current: OpenedWindow) => current.presentative_monitor == monitorInfo.index);
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
          hidden: shouldBeHidden(settings.hideMode, isActive, isOverlaped),
          delayed,
        })}
      >
        <BackgroundByLayersV2 prefix="taskbar" />
        {[
          ...filteredPinOnLeft.map(ItemByType),
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
          ...filteredPinOnCenter.map(ItemByType),
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
          ...filteredPinOnRight.map(ItemByType),
        ]}
      </Reorder.Group>
    </WithContextMenu>
  );
}

function ItemByType(item: SwItem) {
  if (item.type === SwItemType.Pinned && item.path) {
    if (
      item.execution_command.startsWith('shell:AppsFolder') ||
      item.execution_command.endsWith('.exe')
    ) {
      return <UserApplication key={item.execution_command} item={item} />;
    }
    return <FileOrFolder key={item.execution_command} item={item} />;
  }

  if (item.type === SwItemType.TemporalApp && item.path) {
    return <UserApplication key={item.execution_command} item={item} />;
  }

  if (item.type === SwItemType.Media) {
    return <MediaSession key="media-item" item={item} />;
  }

  if (item.type === SwItemType.Start) {
    return <StartMenu key="start-menu" item={item} />;
  }

  return null;
}
